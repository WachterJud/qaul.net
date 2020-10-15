//! A simple ping service for qaul.net
//!
//! This service isn't actually included in the qaul.net application
//! bundle.  It mainly serves as a demonstration on how to write
//! services for libqaul.  This means that this code should be
//! considered documentation.  If you find anything that is unclear to
//! you, or could be commented better, please send us a patch (or MR).

use qrpc_sdk::{default_socket_path, RpcSocket, Service};
use tracing::{error, info};
use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

struct Ping {
    inner: Service,
}

pub(crate) fn parse_log_level() {
    let filter = EnvFilter::try_from_env("QAUL_LOG")
        .unwrap_or_default()
        .add_directive(LevelFilter::TRACE.into())
        .add_directive("async_std=error".parse().unwrap())
        .add_directive("mio=error".parse().unwrap());

    // Initialise the logger
    fmt().with_env_filter(filter).init();
    info!("Initialised logger: welcome to net.qaul.ping!");
}

#[async_std::main]
async fn main() {
    parse_log_level();

    let mut serv = Service::new(
        "net.qaul.ping",
        1,
        "A simple service that says hello to everybody on the network.",
    );

    let (addr, port) = default_socket_path();
    let id = serv
        .register(match RpcSocket::connect(addr, port).await {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to connect to RPC backend: {}", e);
                std::process::exit(1);
            }
        })
        .await
        .unwrap();

    info!("Received service ID '{}' from qrpc-broker", id);

    async_std::task::sleep(std::time::Duration::from_secs(60)).await;
}
