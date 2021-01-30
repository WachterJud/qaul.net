use async_std::{sync::Arc, task};
use futures::try_join;
use ratman_harness::{Initialize, ThreePoint};
use std::{
    env,
    sync::atomic::{AtomicUsize, Ordering},
};
use {
    libqaul::{users::UserUpdate, Qaul},
    libqaul_http::{stream, HttpServer},
    libqaul_rpc1::Responder,
    qaul_chat::Chat,
    qaul_voice::Voice,
};

use tracing::Level;
use tracing_subscriber::fmt;

#[async_std::main]
async fn main() {
    let _s = fmt()
        .with_env_filter("async_std=warn")
        .with_max_level(Level::WARN)
        .init();

    let assets = env::args().nth(1).unwrap_or("".into());
    let assets_b = assets.clone();

    // Initialize a 3 node local qaul network
    let mut tp = ThreePoint::new().await;
    let names = ["florp", "beegleboop"];
    let count = Arc::new(AtomicUsize::new(0));

    tp.init_with(|_, arc| {
        let count = Arc::clone(&count);
        let q = Qaul::new(arc);
        task::block_on(async {
            let auth = q.users().create("1234").await.unwrap();
            let num = count.fetch_add(1, Ordering::Relaxed);
            q.users()
                .update(auth, UserUpdate::DisplayName(Some(names[num].to_owned())))
                .await
                .unwrap();
        });
        q
    });

    // services for Node A
    let chat_a = Chat::new(Arc::clone(&tp.a())).await.unwrap();
    let voices_a = Voice::new(Arc::clone(&tp.a())).await.unwrap();

    // services for Node B
    let chat_b = Chat::new(Arc::clone(&tp.b())).await.unwrap();
    let voices_b = Voice::new(Arc::clone(&tp.b())).await.unwrap();

    // print information for the user
    println!("Path to static web content: {}", assets);
    println!("Open the UI in your web browser:");
    println!("  Node A: http://127.0.0.1:9900");
    println!("  Node B: http://127.0.0.1:9901");

    // configure the web servers
    let server_a = HttpServer::set_paths(
        assets,
        Responder {
            streamer: stream::setup_streamer(),
            qaul: Arc::clone(tp.a()),
            chat: chat_a,
            voice: voices_a,
        },
    );

    let server_b = HttpServer::set_paths(
        assets_b,
        Responder {
            streamer: stream::setup_streamer(),
            qaul: Arc::clone(tp.b()),
            chat: chat_b,
            voice: voices_b,
        },
    );

    // run the servers
    task::block_on(async move {
        let a = server_a.listen("127.0.0.1:9900");
        let b = server_b.listen("127.0.0.1:9901");
        try_join!(a, b).unwrap();
    });
}
