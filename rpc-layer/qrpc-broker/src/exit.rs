//! Exit hook wrapper around atexit

use crate::Broker;
use lazy_static::lazy_static;
use std::{
    os::raw::c_int,
    panic,
    sync::{Arc, Mutex},
};
use tracing::{error, info};

extern "C" {
    fn atexit(callback: extern "C" fn()) -> c_int;
}

lazy_static! {

    /// Store an arc to the socket here for the callback to use
    static ref SOCKET_ARC: Mutex<Option<Arc<Broker>>> = Mutex::new(None);
}

extern "C" fn c_wrapper() {
    shutdown();
}

fn shutdown() {
    info!("Shutting down server-socket!");
    let s = Arc::clone(&get_arc().sock.inner);
    
    match get_arc().sock.shutdown(s.as_ref()) {
        Some(()) => info!("Success!"),
        None => error!("Failed to close server socket!"),
    }
}

/// Add a shudown hook to this program
pub(crate) fn add_shutdown_hooks(arc: Arc<Broker>) -> Option<()> {
    let mut a = SOCKET_ARC.lock().unwrap();
    *a = Some(arc);
    drop(a);

    // Reply to panics
    panic::set_hook(Box::new(|_| shutdown()));

    // Catch various signals
    // unsafe { signal_hook::register(signal_hook::SIGTERM, || shutdown()) };
    // unsafe { signal_hook::register(signal_hook::SIGKILL, || shutdown()) };

    // Set a normal exit hook
    match unsafe { atexit(c_wrapper) } {
        0 => Some(()),
        _ => None,
    }
}

fn get_arc() -> Arc<Broker> {
    let a = SOCKET_ARC.lock().unwrap();
    Arc::clone(
        a.as_ref()
            .expect("Didn't initialise ARC before calling `get_arc`!"),
    )
}
