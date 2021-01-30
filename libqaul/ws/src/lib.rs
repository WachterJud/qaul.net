//! Websocket accept server

#![doc(html_favicon_url = "https://qaul.net/favicon.ico")]
#![doc(html_logo_url = "https://qaul.net/img/qaul_icon-128.png")]

use libqaul::Qaul;
use libqaul_rpc1::{
    json::{RequestEnv, ResponseEnv},
    Envelope,
};
// TODO: these will break if we turn features off
use qaul_chat::Chat;
// use qaul_voices::Voices;

use async_std::{
    net::{TcpListener, TcpStream},
    sync::Arc,
    task,
};
use async_tungstenite::tungstenite::Message;
use futures::prelude::*;
use serde_json;
use std::sync::atomic::{AtomicBool, Ordering};

mod stream;
pub use stream::StreamResp;

pub type Responder = libqaul_rpc1::Responder<StreamResp>;

/// Websocket server structure
pub struct WsServer {
    running: AtomicBool,
    addr: String,

    rpc: Responder,
}

impl WsServer {
    /// Create a websocket server with a libqaul instance and services
    pub fn new<S: Into<String>>(
        addr: S,
        qaul: Arc<Qaul>,
        chat: Arc<Chat>,
        //         voices: Arc<Voices>,
    ) -> Arc<Self> {
        Arc::new(Self {
            running: AtomicBool::from(true),
            addr: addr.into(),
            rpc: Responder {
                streamer: stream::setup_streamer(),
                qaul,
                chat,
            },
        })
    }

    /// Accept connections in a detached task
    pub fn run(self: Arc<Self>) {
        task::spawn(async move {
            while self.running.load(Ordering::Relaxed) {
                println!("Binding '{}'", &self.addr);
                let socket = TcpListener::bind(&self.addr)
                    .await
                    .expect(&format!("Failed to bind; '{}'", &self.addr));

                while let Ok((stream, _)) = socket.accept().await {
                    task::spawn(Arc::clone(&self).handle(stream));
                }
            }
        });
    }

    /// Same as `run` but blocks the current thread
    pub fn block(self: Arc<Self>) {
        task::block_on(async move {
            while self.running.load(Ordering::Relaxed) {
                println!("Binding '{}'", &self.addr);
                let socket = TcpListener::bind(&self.addr)
                    .await
                    .expect(&format!("Failed to bind; '{}'", &self.addr));

                while let Ok((stream, _)) = socket.accept().await {
                    task::spawn(Arc::clone(&self).handle(stream));
                }
            }
        })
    }

    /// Handle an incoming websocket stream
    async fn handle(self: Arc<Self>, stream: TcpStream) {
        let ws_stream = async_tungstenite::accept_async(stream)
            .await
            .expect("Failed ws handshake");

        let (mut tx, mut rx) = ws_stream.split();

        // Read messages from this stream
        while let Some(Ok(Message::Text(msg))) = rx.next().await {
            let req_env: RequestEnv = serde_json::from_str(&msg).expect("Malformed json envelope");
            let Envelope { id, data: req } = match req_env.clone().generate_envelope() {
                Ok(env) => env,
                Err(e) => {
                    tx.send(Message::Text(e))
                        .await
                        .expect("Failed to send error message!");
                    continue;
                }
            };

            // Call into libqaul via the rpc utilities
            let resp = self.rpc.respond(req).await;
            let env = Envelope { id, data: resp };

            // Build the reply envelope
            let resp_env: ResponseEnv = (env, req_env).into();
            let json = serde_json::to_string(&resp_env).unwrap();

            // Send the reply
            tx.send(Message::Text(json))
                .await
                .expect("Failed to send reply!");

            // Break on server shutdown
            // The if is here because of a possible rustc bug and does nothing
            if !self.running.load(Ordering::Relaxed) && break {};
        }
    }

    /// Signal the runner to shut down
    pub fn stop(&self) {
        self.running.swap(false, Ordering::Relaxed);
    }
}
