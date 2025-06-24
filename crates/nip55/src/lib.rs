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

extern crate ndk;

use std::sync::Arc;

use jni::objects::{JObject, JValue};
use jni::sys::jobject;
use jni::{JNIEnv, JavaVM};
use ndk_context::android_context;
use nostr::event::{Event, UnsignedEvent};
use nostr::JsonUtil;

/// Signer for interaction with Android signers (i.e., Amber)
///
/// <https://github.com/nostr-protocol/nips/blob/master/55.md>
#[derive(Debug, Clone)]
pub struct AndroidSigner {
    package_name: String,
    jvm: Arc<JavaVM>,
}

impl AndroidSigner {
    pub fn new(package_name: String, jvm: JavaVM) -> Self {
        Self {
            package_name,
            jvm: Arc::new(jvm),
        }
    }

    // Send intent to Android signer
    pub fn send_sign_request(
        &self,
        unsigned: &UnsignedEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut env: JNIEnv = self.jvm.attach_current_thread_as_daemon()?;

        // Get Android context
        let context: jobject = android_context().context() as jobject;
        let context_obj: JObject = JObject::from(context);

        // Create Intent for NIP-55 communication
        self.create_sign_intent(&mut env, &context_obj, unsigned)
    }

    fn create_sign_intent(
        &self,
        env: &mut JNIEnv,
        context: &JObject,
        unsigned: &UnsignedEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create Intent with NIP-55 action
        let intent_class = env.find_class("android/content/Intent")?;
        let intent = env.new_object(intent_class, "()V", &[])?;

        // Set action for NIP-55 signer
        let action = env.new_string("android.intent.action.VIEW")?;
        env.call_method(
            &intent,
            "setAction",
            "(Ljava/lang/String;)Landroid/content/Intent;",
            &[JValue::Object(&action)],
        )?;

        // Create URI for NIP-55: nostrsigner:event?...
        let uri_string = format!("nostrsigner:{}", unsigned.as_json());
        let uri_jstring = env.new_string(&uri_string)?;

        // Parse URI
        let uri_class = env.find_class("android/net/Uri")?;
        let uri = env.call_static_method(
            uri_class,
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(&uri_jstring)],
        )?;

        // Set data URI
        env.call_method(
            &intent,
            "setData",
            "(Landroid/net/Uri;)Landroid/content/Intent;",
            &[uri.borrow()],
        )?;

        // Add package name for specific signer (optional)
        let package_name = env.new_string(&self.package_name)?;
        env.call_method(
            &intent,
            "setPackage",
            "(Ljava/lang/String;)Landroid/content/Intent;",
            &[JValue::Object(&package_name)],
        )?;

        // Start activity with intent
        env.call_method(
            context,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[JValue::Object(&intent)],
        )?;

        Ok(())
    }

    // Handle incoming intent with signed event
    pub fn handle_sign_intent_response(
        &self,
        intent_uri: &str,
    ) -> Result<Event, Box<dyn std::error::Error>> {
        // Parse the nostrsigner response URI
        if intent_uri.starts_with("nostrsigner:") {
            let response_data = &intent_uri[12..]; // Remove "nostrsigner:" prefix

            // Parse the signed event JSON
            let event: Event = Event::from_json(response_data)?;

            event.verify()?;

            Ok(event)
        } else {
            Err("Invalid nostrsigner response URI".into())
        }
    }
}
