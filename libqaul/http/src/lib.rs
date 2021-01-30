//! libqaul http web server
//!
//! The web server serves the following things
//!
//! * the static files of the EmberJS webGUI
//! * the HTTP API for the webGUI
//! * the RPC API

#![doc(html_favicon_url = "https://qaul.net/favicon.ico")]
#![doc(html_logo_url = "https://qaul.net/img/qaul_icon-128.png")]

use async_std::{sync::Arc, task, task::JoinHandle};
use std::io::Result;

use tide::{self, server::Server};
use tide_naive_static_files::StaticFilesEndpoint as StaticEp;

mod http;
mod rpc;
pub mod stream;

pub(crate) use stream::StreamResp;

/// An http specific responder type
///
/// This type hides generics on the Responder type to make it easier
/// to initialise the streaming context for Http purposes.
pub type Responder = libqaul_rpc1::Responder<StreamResp>;

/// State structure for the libqaul http server
pub struct HttpServer {
    inner: Server<()>,
}

impl HttpServer {
    /// open a blocking http connection
    pub fn block(addr: &str, path: String, rpc: Responder) {
        let Self { inner } = HttpServer::set_paths(path, rpc);

        // run server in blocking task
        task::block_on(async move { inner.listen(addr).await }).unwrap();
    }

    pub fn listen(self, addr: &str) -> JoinHandle<Result<()>> {
        let Self { inner } = self;
        let addr = String::from(addr);
        task::spawn(async move { inner.listen(&addr).await })
    }

    /// set http endpoints and paths that returns the http server
    pub fn set_paths(path: String, rpc: Responder) -> Self {
        let mut app = tide::new();
        let rpc_state = Arc::new(rpc);
        let http_state = rpc_state.clone();

        // REST Endpoint
        app.at("/http")
            .strip_prefix()
            .nest(http::routes::http_routes(http_state));

        // RPC Endpoint
        app.at("/rpc")
            .strip_prefix()
            .nest(rpc::rpc_routes(rpc_state));

        // static file handler for the webui, assumes the webui exists
        let fav_path = path.clone();
        let mut assets_path = path.clone();
        assets_path.push_str("/assets");
        let feed_path = path.clone();
        let feed_path_2 = path.clone();
        let messenger_path = path.clone();
        let messenger_path_2 = path.clone();
        let users_path = path.clone();
        let users_path_2 = path.clone();
        let files_path = path.clone();
        let files_path_2 = path.clone();
        let settings_path = path.clone();
        let settings_path_2 = path.clone();
        let info_path = path.clone();
        let info_path_2 = path.clone();
        let login_path = path.clone();

        app.at("/").get(StaticEp { root: path.into() });
        app.at("/favicon.ico").get(StaticEp {
            root: fav_path.into(),
        });
        app.at("/assets/").strip_prefix().get(StaticEp {
            root: assets_path.into(),
        });
        // WebGUI virtual routes
        app.at("/feed").strip_prefix().get(StaticEp {
            root: feed_path.into(),
        });
        app.at("/feed/*").strip_prefix().get(StaticEp {
            root: feed_path_2.into(),
        });
        app.at("/messenger").strip_prefix().get(StaticEp {
            root: messenger_path.into(),
        });
        app.at("/messenger/*").strip_prefix().get(StaticEp {
            root: messenger_path_2.into(),
        });
        app.at("/users").strip_prefix().get(StaticEp {
            root: users_path.into(),
        });
        app.at("/users/*").strip_prefix().get(StaticEp {
            root: users_path_2.into(),
        });
        app.at("/files").strip_prefix().get(StaticEp {
            root: files_path.into(),
        });
        app.at("/files/*").strip_prefix().get(StaticEp {
            root: files_path_2.into(),
        });
        app.at("/settings").strip_prefix().get(StaticEp {
            root: settings_path.into(),
        });
        app.at("/settings/*").strip_prefix().get(StaticEp {
            root: settings_path_2.into(),
        });
        app.at("/info").strip_prefix().get(StaticEp {
            root: info_path.into(),
        });
        app.at("/info/*").strip_prefix().get(StaticEp {
            root: info_path_2.into(),
        });
        app.at("/login").strip_prefix().get(StaticEp {
            root: login_path.into(),
        });

        Self { inner: app }
    }
}
