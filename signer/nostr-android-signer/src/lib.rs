// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Android signer implementation (NIP-55).
//!
//! <https://github.com/nostr-protocol/nips/blob/master/55.md>

#![cfg_attr(test, allow(missing_docs))]
#![cfg_attr(not(test), warn(missing_docs))]
#![warn(rustdoc::bare_urls)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use jni::objects::{JClass, JMethodID, JObject, JStaticMethodID, JString, JValue, JValueOwned};
use jni::signature::{Primitive, ReturnType};
use jni::sys::{jint, jmethodID, jobject, JavaVM as RawJavaVM, JNI_OK};
use jni::{JNIEnv, JavaVM};
use jvm_getter::JNI_GetCreatedJavaVMs;
use once_cell::sync::{Lazy, OnceCell};

pub mod error;
pub mod types;

use self::error::Error;
use self::types::Permission;

const FLAG_ACTIVITY_NEW_TASK: jint = 0x10000000;
// FLAG_ACTIVITY_SINGLE_TOP = 0x20000000
// FLAG_ACTIVITY_CLEAR_TOP = 0x04000000
const CONTENT_RESOLVER_TIMEOUT: Duration = Duration::from_secs(30);
const POLLING_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub struct ActivityResult {
    pub request_code: i32,
    pub result_code: i32,
    pub data: Option<JObject<'static>>,
}

#[derive(Clone, Copy)]
enum Request {
    GetPublicKey,
}

impl Request {
    fn as_str_for_content_resolver(&self) -> &str {
        match self {
            Request::GetPublicKey => "GET_PUBLIC_KEY",
        }
    }
}

#[derive(Debug)]
struct AndroidContext {
    jvm: JavaVM,
    ctx: OnceCell<JObject<'static>>,
    //activity: OnceCell<JObject<'static>>,
}

impl AndroidContext {
    fn new() -> Result<Self, Error> {
        Ok(Self {
            jvm: get_jvm()?,
            ctx: OnceCell::new(),
            //activity: OnceCell::new(),
        })
    }

    /// Get JVM env
    #[inline]
    fn get_env(&self) -> Result<JNIEnv, Error> {
        Ok(self.jvm.attach_current_thread_as_daemon()?)
    }

    /// Get android context
    fn get_context(&self) -> Result<&JObject<'static>, Error> {
        self.ctx.get_or_try_init(|| {
            let mut env: JNIEnv = self.get_env()?;
            get_global_android_context(&mut env)
        })
    }
}

/// Signer for interaction with Android signers (i.e., Amber)
///
/// <https://github.com/nostr-protocol/nips/blob/master/55.md>
#[derive(Debug, Clone)]
pub struct AndroidSigner {
    ctx: Arc<AndroidContext>,
    package_name: OnceCell<String>,
    request_code: Arc<AtomicUsize>,
}

impl AndroidSigner {
    /// New android signer
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            ctx: Arc::new(AndroidContext::new()?),
            package_name: OnceCell::new(),
            request_code: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Get the next request code
    fn next_request_code(&self) -> usize {
        self.request_code.fetch_add(1, Ordering::SeqCst)
    }

    #[inline]
    pub fn set_package_name(&self, package_name: &str) -> Result<(), Error> {
        self.package_name
            .set(package_name.to_string())
            .map_err(|_| Error::PackageNameAlreadySet)
    }

    /// Check if an external signer is installed on the device
    pub fn is_external_signer_installed(&self) -> Result<bool, Error> {
        let mut env: JNIEnv = self.ctx.get_env()?;
        let context: &JObject = self.ctx.get_context()?;

        // Create Intent
        let intent: JObject = create_intent(&mut env)?;

        // Set action to ACTION_VIEW
        set_intent_action_view(&mut env, &intent)?;

        // Parse URI
        let uri: JValueOwned = parse_uri(&mut env, "nostrsigner:")?;

        // Set data URI
        set_intent_data(&mut env, &intent, uri.borrow())?;

        // Get PackageManager
        let package_manager = env.call_method(
            context,
            "getPackageManager",
            "()Landroid/content/pm/PackageManager;",
            &[],
        )?;

        // Query intent activities
        let activities = env.call_method(
            package_manager.l()?,
            "queryIntentActivities",
            "(Landroid/content/Intent;I)Ljava/util/List;",
            &[JValue::Object(&intent), JValue::Int(0)],
        )?;

        // Get the size of the list
        let size = env.call_method(activities.l()?, "size", "()I", &[])?;

        // Return true if there are any activities that can handle the intent
        Ok(size.i()? > 0)
    }

    fn launch_get_public_key_intent(
        &self,
        permissions: Option<Vec<Permission>>,
    ) -> Result<(), Error> {
        let mut env: JNIEnv = self.ctx.get_env()?;

        let context: &JObject = self.ctx.get_context()?;

        // Create Intent
        let intent: JObject = create_intent(&mut env)?;

        // Set action to ACTION_VIEW
        set_intent_action_view(&mut env, &intent)?;

        // Parse URI
        let uri: JValueOwned = parse_uri(&mut env, "nostrsigner:")?;

        // Set data URI
        set_intent_data(&mut env, &intent, uri.borrow())?;

        // Add type extra
        intent_put_extra(&mut env, &intent, "type", "get_public_key")?;

        // Add permissions if provided
        if let Some(permissions) = permissions {
            let permissions_json: String = serde_json::to_string(&permissions)?;
            intent_put_extra(&mut env, &intent, "permissions", &permissions_json)?;
        }

        // Add intent flags for multiple intents
        add_intent_flags(&mut env, &intent, FLAG_ACTIVITY_NEW_TASK)?;

        // Start activity
        start_activity(&mut env, context, &intent)?;
        //let request_code = self.next_request_code();
        //start_activity_for_result(&mut env, context, &intent, request_code as i32)?;

        Ok(())
    }

    /// Get public key from signer using Content Resolver
    pub fn public_key(&self, permissions: Option<Vec<Permission>>) -> Result<(), Error> {
        let package_name = self.package_name.get().ok_or(Error::PackageNameNotSet)?;

        //let (tx, rx) = mpsc::channel();

        self.launch_get_public_key_intent(permissions)?;

        // // Wait for response
        // match rx.recv_timeout(Duration::from_secs(60)) {
        //     Ok(res) => println!("Got response: {:?}", res),
        //     Err(e) => panic!("Error: {:?}", e),
        // }
        let res = self.wait_for_response(package_name, Request::GetPublicKey)?;
        println!("Got response: {:?}", res);

        Ok(())
    }

    /// Wait for response from signer using Content Resolver
    fn wait_for_response(&self, package_name: &str, req: Request) -> Result<String, Error> {
        let mut env: JNIEnv = self.ctx.get_env()?;
        let context: &JObject = self.ctx.get_context()?;
    
        let start_time = Instant::now();
    
        loop {
            // Check if timeout exceeded
            if start_time.elapsed() > CONTENT_RESOLVER_TIMEOUT {
                return Err(Error::Timeout);
            }
    
            // Try to get response from content resolver
            if let Ok(response) = query_content_resolver(&mut env, context, package_name, req) {
                if !response.is_empty() {
                    return Ok(response);
                }
            }
    
            // Wait before next poll
            std::thread::sleep(POLLING_INTERVAL);
        }
    }
}

/// Get JVM
fn get_jvm() -> Result<JavaVM, Error> {
    let jni_get_created_java_vms: JNI_GetCreatedJavaVMs =
        unsafe { jvm_getter::find_jni_get_created_java_vms().ok_or(Error::JVMNotFound)? };

    let mut vm: MaybeUninit<*mut RawJavaVM> = MaybeUninit::uninit();
    let status: jint = unsafe { jni_get_created_java_vms(vm.as_mut_ptr(), 1, &mut 0) };
    if status != JNI_OK {
        return Err(Error::JVMNotFound);
    }

    Ok(unsafe { JavaVM::from_raw(vm.assume_init())? })
}

/// Get global android context
fn get_global_android_context(env: &mut JNIEnv) -> Result<JObject<'static>, Error> {
    // Snippet from https://stackoverflow.com/a/46871051
    //
    // static jobject getGlobalContext(JNIEnv *env)
    // {
    //     jclass activityThread = (*env)->FindClass(env,"android/app/ActivityThread");
    //     jmethodID currentActivityThread = (*env)->GetStaticMethodID(env, activityThread, "currentActivityThread", "()Landroid/app/ActivityThread;");
    //     jobject activityThreadObj = (*env)->CallStaticObjectMethod(env, activityThread, currentActivityThread);
    //
    //     jmethodID getApplication = (*env)->GetMethodID(env, activityThread, "getApplication", "()Landroid/app/Application;");
    //     jobject context = (*env)->CallObjectMethod(env, activityThreadObj, getApplication);
    //     return context;
    // }

    let activity_thread: JClass = env.find_class("android/app/ActivityThread")?;
    let current_activity_thread: JStaticMethodID = env.get_static_method_id(
        &activity_thread,
        "currentActivityThread",
        "()Landroid/app/ActivityThread;",
    )?;

    let activity_thread_obj: JValueOwned = unsafe {
        env.call_static_method_unchecked(
            &activity_thread,
            current_activity_thread,
            ReturnType::Object,
            &[],
        )?
    };

    // Get the getApplication method
    let get_application_method: JMethodID = env.get_method_id(
        &activity_thread,
        "getApplication",
        "()Landroid/app/Application;",
    )?;

    // Call getApplication method to get the context
    let context: JValueOwned = unsafe {
        env.call_method_unchecked(
            activity_thread_obj.l()?,
            get_application_method,
            ReturnType::Object,
            &[],
        )?
    };

    // Get context object
    let raw: jobject = context.l()?.as_raw();
    Ok(unsafe { JObject::from_raw(raw) })
}

/// Get the current Activity context (not Application context)
fn get_current_activity_context(env: &mut JNIEnv) -> Result<JObject<'static>, Error> {
    // Get ActivityThread
    let activity_thread_class: JClass = env.find_class("android/app/ActivityThread")?;

    let current_activity_thread: JStaticMethodID = env.get_static_method_id(
        &activity_thread_class,
        "currentActivityThread",
        "()Landroid/app/ActivityThread;",
    )?;

    let activity_thread_obj: JValueOwned = unsafe {
        env.call_static_method_unchecked(
            &activity_thread_class,
            current_activity_thread,
            ReturnType::Object,
            &[],
        )?
    };

    // Try to get current activity using reflection
    // ActivityThread.currentActivityThread().mActivities or similar
    let activity_thread = activity_thread_obj.l()?;

    // Get mActivities field (ArrayMap of activities)
    let activities_field = env.get_field_id(
        &activity_thread_class,
        "mActivities",
        "Landroid/util/ArrayMap;",
    )?;
    let activities_map =
        env.get_field_unchecked(activity_thread, activities_field, ReturnType::Object)?;

    // Get the first activity from the map
    let array_map = activities_map.l()?;
    if array_map.is_null() {
        return Err(Error::ContentResolver("No activities found".to_string()));
    }

    // Get size of map
    let size = env.call_method(&array_map, "size", "()I", &[])?;
    if size.i()? == 0 {
        return Err(Error::ContentResolver("No activities in map".to_string()));
    }

    // Get first activity record
    let activity_record = env.call_method(
        array_map,
        "valueAt",
        "(I)Ljava/lang/Object;",
        &[JValue::Int(0)],
    )?;
    let activity_record_obj = activity_record.l()?;

    // Get the activity from the record (ActivityClientRecord.activity)
    let record_class = env.get_object_class(&activity_record_obj)?;
    let activity_field = env.get_field_id(&record_class, "activity", "Landroid/app/Activity;")?;
    let activity =
        env.get_field_unchecked(activity_record_obj, activity_field, ReturnType::Object)?;

    let activity_obj = activity.l()?;
    if activity_obj.is_null() {
        return Err(Error::ContentResolver("Activity is null".to_string()));
    }

    let raw: jobject = activity_obj.as_raw();
    Ok(unsafe { JObject::from_raw(raw) })
}

fn create_intent<'a>(env: &mut JNIEnv<'a>) -> Result<JObject<'a>, Error> {
    let intent_class = env.find_class("android/content/Intent")?;
    Ok(env.new_object(intent_class, "()V", &[])?)
}

/// Set action to ACTION_VIEW
fn set_intent_action_view<'a>(env: &mut JNIEnv<'a>, intent: &JObject<'a>) -> Result<(), Error> {
    let action: JString = env.new_string("android.intent.action.VIEW")?;
    env.call_method(
        intent,
        "setAction",
        "(Ljava/lang/String;)Landroid/content/Intent;",
        &[JValue::Object(&action)],
    )?;
    Ok(())
}

fn set_intent_data(env: &mut JNIEnv, intent: &JObject, value: JValue) -> Result<(), Error> {
    env.call_method(
        intent,
        "setData",
        "(Landroid/net/Uri;)Landroid/content/Intent;",
        &[value],
    )?;
    Ok(())
}

fn set_intent_package(env: &mut JNIEnv, intent: &JObject, package: &str) -> Result<(), Error> {
    let package_jstring: JString = env.new_string(package)?;
    env.call_method(
        intent,
        "setPackage",
        "(Ljava/lang/String;)Landroid/content/Intent;",
        &[JValue::Object(&package_jstring)],
    )?;
    Ok(())
}

fn intent_put_extra(
    env: &mut JNIEnv,
    intent: &JObject,
    key: &str,
    value: &str,
) -> Result<(), Error> {
    let key_jstring: JString = env.new_string(key)?;
    let value_jstring: JString = env.new_string(value)?;
    env.call_method(
        intent,
        "putExtra",
        "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
        &[JValue::Object(&key_jstring), JValue::Object(&value_jstring)],
    )?;
    Ok(())
}

fn string_to_jobject<'a>(env: &mut JNIEnv, data: &'a str) -> Result<JObject<'a>, Error> {
    let jstring: JString = env.new_string(data)?;
    let raw = jstring.as_raw();
    unsafe { Ok(JObject::from_raw(raw)) }
}

fn add_intent_flags(env: &mut JNIEnv, intent: &JObject, flags: jint) -> Result<(), Error> {
    env.call_method(
        intent,
        "addFlags",
        "(I)Landroid/content/Intent;",
        &[JValue::Int(flags)],
    )?;
    Ok(())
}

fn parse_uri<'a>(env: &mut JNIEnv<'a>, uri: &str) -> Result<JValueOwned<'a>, Error> {
    let uri_jstring: JString = env.new_string(uri)?;

    // Parse URI
    let uri_class: JClass = env.find_class("android/net/Uri")?;
    Ok(env.call_static_method(
        uri_class,
        "parse",
        "(Ljava/lang/String;)Landroid/net/Uri;",
        &[JValue::Object(&uri_jstring)],
    )?)
}

fn start_activity(env: &mut JNIEnv, context: &JObject, intent: &JObject) -> Result<(), Error> {
    env.call_method(
        context,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(intent)],
    )?;
    Ok(())
}

fn start_activity_for_result(
    env: &mut JNIEnv,
    context: &JObject,
    intent: &JObject,
    request_code: i32,
) -> Result<(), Error> {
    env.call_method(
        context,
        "startActivityForResult",
        "(Landroid/content/Intent;I)V",
        &[JValue::Object(intent), JValue::Int(request_code)],
    )?;

    Ok(())
}

/// Create a string array for JNI
fn create_string_array<'a>(env: &mut JNIEnv<'a>, strings: &[&str]) -> Result<JObject<'a>, Error> {
    let string_class = env.find_class("java/lang/String")?;
    let array = env.new_object_array(strings.len() as i32, string_class, JObject::null())?;

    for (i, string) in strings.iter().enumerate() {
        let jstring = env.new_string(string)?;
        env.set_object_array_element(&array, i as i32, jstring)?;
    }

    Ok(array.into())
}

fn get_column_index<'a>(env: &mut JNIEnv<'a>, cursor: &JObject, column: &str) -> Result<JValueOwned<'a>, Error> {
    let obj = string_to_jobject(env, column)?;
    Ok(env.call_method(
        cursor,
        "getColumnIndex",
        "(Ljava/lang/String;)I",
        &[JValue::Object(&obj)],
    )?)
}

/// Query the signer's content resolver for response
fn query_content_resolver(
    env: &mut JNIEnv,
    context: &JObject,
    package_name: &str,
    req: Request,
) -> Result<String, Error> {
    // Get ContentResolver
    let content_resolver = env.call_method(
        context,
        "getContentResolver",
        "()Landroid/content/ContentResolver;",
        &[],
    )?;

    // Build URI for the content provider
    let uri_string = format!(
        "content://{package_name}/{}",
        req.as_str_for_content_resolver()
    );

    let uri = parse_uri(env, &uri_string)?;

    // Define projection (columns to query)
    let projection = create_string_array(env, &["login"])?;

    // Query the content provider
    let cursor = env.call_method(
        content_resolver.l()?,
        "query",
        "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;",
        &[
            uri.borrow(),
            JValue::Object(&projection),
            JValue::Object(&JObject::null()),
            JValue::Object(&JObject::null()),
            JValue::Object(&JObject::null()),
        ],
    )?;

    let cursor_obj = cursor.l()?;

    // Check if cursor is null
    if cursor_obj.is_null() {
        return Err(Error::ContentResolver("Cursor is null".to_string()));
    }

    // Check if request was rejected
    let rejected_index = get_column_index(env, &cursor_obj, "rejected")?;

    if rejected_index.i()? > -1 {
        env.call_method(&cursor_obj, "close", "()V", &[])?;
        return Err(Error::RequestRejected);
    }

    // Move to first row
    let has_data = env.call_method(&cursor_obj, "moveToFirst", "()Z", &[])?;

    if !has_data.z()? {
        // Close cursor
        env.call_method(&cursor_obj, "close", "()V", &[])?;
        return Err(Error::ContentResolver("No data found".to_string()));
    }

    // Get result column index
    let result_index = get_column_index(env, &cursor_obj, "result")?;
    let result_index: jint = result_index.i()?;

    if result_index < 0 {
        env.call_method(cursor_obj, "close", "()V", &[])?;
        return Err(Error::ContentResolver(
            "Result column not found".to_string(),
        ));
    }

    // Get public key
    let pubkey_result = env.call_method(
        &cursor_obj,
        "getString",
        "(I)Ljava/lang/String;",
        &[JValue::Int(result_index)],
    )?;
    let pubkey_result = pubkey_result.l()?;

    let result = if !pubkey_result.is_null() {
        let pubkey_jstring: JString = pubkey_result.into();
        let pubkey_str: String = env.get_string(&pubkey_jstring)?.into();
        pubkey_str
    } else {
        panic!("Public key is null");
    };

    // Close cursor
    env.call_method(cursor_obj, "close", "()V", &[])?;

    Ok(result)
}
