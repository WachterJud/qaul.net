//! qaul.net android interop library

#![cfg(target_os = "android")]
#![allow(warnings)]

#[macro_use]
extern crate log;
extern crate android_logger;

pub mod api;
mod utils;

// use async_std::{
//     sync::RwLock,
//     task::{block_on, spawn},
// };
// use jni::objects::{JList, JObject, JString, JValue};
// use jni::sys::{jboolean, jint, jlong, jobject, jcharArray};
// use jni::JNIEnv;
// use std::{
//     ffi::{CStr, CString},
//     sync::Arc,
// };

// use android_logger::{Config, FilterBuilder};
// use log::Level;

// use libqaul::{users::UserAuth, Qaul};
// use libqaul_http::{stream, HttpServer};
// use libqaul_rpc1::Responder;
// use qaul_chat::Chat;
// use qaul_voice::Voice;
// use ratman_configure::{EpBuilder, NetBuilder};

// use tracing_subscriber::{layer::SubscriberExt, EnvFilter};



// #[no_mangle]
// pub unsafe extern "C" fn Java_net_qaul_app_ffi_NativeQaul_wdReceiveFrame<'env>(
//     env: JNIEnv<'env>,
//     _: JObject,
//     qaul: jlong,
//     target: jint,
//     data: jchatArray,
// ){
    
// }


