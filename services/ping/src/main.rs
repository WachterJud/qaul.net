//! A simple ping service for qaul.net
//!
//! This service isn't actually included in the qaul.net application
//! bundle.  It mainly serves as a demonstration on how to write
//! services for libqaul.  This means that this code should be
//! considered documentation.  If you find anything that is unclear to
//! you, or could be commented better, please send us a patch (or MR).

use qrpc_sdk::{default_socket_path, RpcSocket, Service};

struct Ping {
    inner: Service,
}

#[async_std::main]
async fn main() {
    let mut serv = Service::new(
        "net.qaul.ping",
        1,
        "A simple service that says hello to everybody on the network.",
    );
    let sock = RpcSocket::new(default_socket_path()).unwrap();

    let sock2 = RpcSocket::new(default_socket_path()).unwrap();

    let sock3 = RpcSocket::new(default_socket_path()).unwrap();
//     serv.register(sock).await.unwrap();
}
