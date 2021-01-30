#![doc(html_favicon_url = "https://qaul.net/favicon.ico")]
#![doc(html_logo_url = "https://qaul.net/img/qaul_icon-128.png")]

use async_std::sync::Arc;
use ratman::Router;
use std::{env, process};
use {
    libqaul::Qaul,
    libqaul_http::{stream, HttpServer},
    libqaul_rpc1::Responder,
    qaul_chat::Chat,
    qaul_voice::Voice,
};

#[async_std::main]
async fn main() {
    let assets = match env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("Usage: linux-http-test <path-to-static-webgui-directory>");
            process::exit(2);
        }
    };

    // Init a basic libqaul stack with no interfaces
    let rat = Router::new();
    let qaul = Qaul::new(rat);
    let chat = Chat::new(Arc::clone(&qaul)).await.unwrap();
    let voice = Voice::new(Arc::clone(&qaul)).await.unwrap();

    // print the path to the static
    println!("Path to static web content: {}", assets);
    println!("Open http://127.0.0.1:9900 in your web browser");

    // Start the websocket server
    HttpServer::block(
        "127.0.0.1:9900",
        assets,
        Responder {
            streamer: stream::setup_streamer(),
            qaul,
            chat,
            voice,
        },
    );
}
