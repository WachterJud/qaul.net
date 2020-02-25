//! Websocket accept server

use crate::env::{RequestEnv, ResponseEnv};

use async_std::{
    net::{TcpListener, TcpStream},
    sync::Arc,
    task,
};
use async_tungstenite::tungstenite::Message;
use futures::prelude::*;
use libqaul_rpc::{Envelope, EnvelopeType, Responder};
use serde_json;
use std::sync::atomic::{AtomicBool, Ordering};

/// Websocket server structure
pub struct WsServer {
    running: AtomicBool,
    addr: String,

    rpc: Responder,
}

impl WsServer {
    /// Create a websocket server with a libqaul instance and services
    #[cfg(all(feature = "chat", feature = "voice", feature = "files"))]
    pub fn new<S: Into<String>>(addr: S, qaul: Arc<Qaul>, chat: Arc<Chat>) -> Arc<Self> {
        Arc::new(Self {
            running: AtomicBool::from(true),
            addr: addr.into(),
            qaul,
            chat,
        })
    }

    /// Accept connections in a detached task
    pub fn run(self: Arc<Self>) {
        task::spawn(async move {
            while self.running.load(Ordering::Relaxed) {
                let socket = TcpListener::bind(&self.addr)
                    .await
                    .expect(&format!("Failed to bind; '{}'", &self.addr));

                while let Ok((stream, _)) = socket.accept().await {
                    task::spawn(Arc::clone(&self).handle(stream));
                }
            }
        });
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
            let Envelope { id, data } = req_env.clone().into();

            let req = match data {
                EnvelopeType::Request(req) => req,
                _ => unreachable!(), // Obviously possibly but fuck you
            };

            // Call into libqaul via the rpc utilities
            let resp = self.rpc.respond(req).await;
            let env = Envelope {
                id,
                data: EnvelopeType::Response(resp),
            };

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

        // while let Ok(num) = rx.read_to_string(&mut buf).await {

        // }
    }

    /// Signal the runner to shut down
    pub fn stop(&self) {
        self.running.swap(false, Ordering::Relaxed);
    }
}
