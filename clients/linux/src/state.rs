//! Manage the libqaul, service and ratman states

use crate::cfg::Config;
use directories::ProjectDirs;
use netmod_tcp::{Endpoint, Mode};
use ratman::Router;
use std::collections::HashSet;
use std::{fs::File, io::Read, net::SocketAddr, str::FromStr, sync::Arc};
use async_std::{task, task::spawn};
use {
    libqaul::{users::UserUpdate, Qaul},
    libqaul_http::{stream, HttpServer},
    libqaul_rpc1::Responder,
    qaul_chat::Chat,
    qaul_voice::Voice,
};

#[allow(unused)]
pub(crate) struct State {
    qaul: Arc<Qaul>,
    router: Arc<Router>,
}

impl State {
    /// Create a new run state
    pub(crate) async fn new(cfg: &Config) -> State {
        let ep = Endpoint::new(
            &cfg.addr,
            cfg.port,
            "qaul-linux",
            match cfg.mode.as_str() {
                "dynamic" => Mode::Dynamic,
                _ => Mode::Static,
            },
        )
        .await
        .unwrap();

        let mut buf = String::new();
        let mut peersfd = File::open(&cfg.peers).unwrap();
        peersfd.read_to_string(&mut buf).unwrap();

        let peers = buf.split("\n").map(|s| s.to_string()).collect();
        ep.add_peers(peers).await.unwrap();

        let router = Router::new();
        router.add_endpoint(ep).await;

        let dirs = ProjectDirs::from("net", "qaul", "hubd").unwrap();
        let qaul = Qaul::new(Arc::clone(&router));

        // services
        let chat = Chat::new(Arc::clone(&qaul)).await.unwrap();
        let voices = Voice::new(Arc::clone(&qaul)).await.unwrap();

        // print information for the user
        println!("Path to static web content: {}", cfg.webgui);
        println!("Open the UI in your web browser:");
        println!("WebGUI: http://127.0.0.1:9900");

        // configure the web servers
        let server = HttpServer::set_paths(
            cfg.webgui.clone(),
            Responder {
                streamer: stream::setup_streamer(),
                qaul: Arc::clone(&qaul),
                chat: chat,
                voice: voices,
            },
        );

        task::spawn(async move {
            let http = server.listen("127.0.0.1:9900");
        });
        /*
        // run the servers
        task::block_on(async move {
            let a = server_a.listen("127.0.0.1:9900");
            let b = server_b.listen("127.0.0.1:9901");
            try_join!(a, b).unwrap();
        });
*/

        Self { qaul, router }
    }
}
