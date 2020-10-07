//! # qaul-hubd server
//!
//! A modular and configurable internet overlay server for qaul.net.

mod cfg;
mod log;
mod state;
mod upnp;

use async_std::{future, task, task::Poll};
use state::State;
use std::time::Duration;

use tracing::error;

pub(crate) fn elog<S: Into<String>>(msg: S, code: u16) -> ! {
    tracing::error!("{}", msg.into());
    std::process::exit(code.into());
}

use qrpc_sdk::RpcSocket;

#[async_std::main]
async fn main() {
    log::parse_log_level();

    // let app = cfg::cli();
    // let cfg = cfg::match_fold(app);
    // let _state = State::new(&cfg).await;

    // // !no_upnp means upnp has _not_ been disabled
    // if !cfg.no_upnp {
    //     if upnp::open_port(cfg.port).is_none() {
    //         error!("Failed to open UPNP port; your router probably doesn't support it...");
    //     }
    // }

    // let _ = future::timeout(Duration::from_secs(10), async {
    //     let _: () = future::poll_fn(|_| Poll::Pending).await;
    // }).await;
}
