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

use std::mem::MaybeUninit;
use std::sync::Arc;

use jni::objects::{JClass, JMethodID, JObject, JStaticMethodID, JString, JValue, JValueOwned};
use jni::signature::ReturnType;
use jni::sys::{jint, jobject, JavaVM as RawJavaVM, JNI_OK};
use jni::{JNIEnv, JavaVM};
use jvm_getter::JNI_GetCreatedJavaVMs;
use once_cell::sync::OnceCell;

pub mod error;

use self::error::Error;

#[derive(Debug)]
struct AndroidContext {
    jvm: JavaVM,
    ctx: OnceCell<JObject<'static>>,
}

impl AndroidContext {
    fn new() -> Result<Self, Error> {
        Ok(Self {
            jvm: get_jvm()?,
            ctx: OnceCell::new(),
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
}

impl AndroidSigner {
    /// New android signer
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            ctx: Arc::new(AndroidContext::new()?),
        })
    }

    /// Check if an external signer is installed on the device
    pub fn is_external_signer_installed(&self) -> Result<bool, Error> {
        let mut env: JNIEnv = self.ctx.get_env()?;
        let context: &JObject = self.ctx.get_context()?;
        check_signer_availability(&mut env, context)
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

fn check_signer_availability(env: &mut JNIEnv, context: &JObject) -> Result<bool, Error> {
    // Create Intent
    let intent: JObject = create_intent(env)?;

    // Set action to ACTION_VIEW
    set_intent_action_view(env, &intent)?;

    // Parse URI
    let uri: JValueOwned = parse_uri(env, "nostrsigner:")?;

    // Set data URI
    set_intent_data(env, &intent, uri.borrow())?;

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
