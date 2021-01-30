//! #RPC tide http endpoint

use crate::Responder;
use async_std::sync::Arc;
use libqaul_rpc1::{
    json::{RequestEnv, ResponseEnv},
    Envelope,
};
use tide::{Request, Response, Server};

/// Creates the tide server and routes for RPC endpoint
pub fn rpc_routes(rpc_state: Arc<Responder>) -> Server<Arc<Responder>> {
    let mut app_rpc = tide::with_state(rpc_state);

    app_rpc.at("/").post(|mut r: Request<Arc<Responder>>| {
        async move {
            let hopefully_json: String = dbg!(r.body_string().await).unwrap();
            let req_env: RequestEnv =
                serde_json::from_str(hopefully_json.as_str()).expect("Malformed json envelope");
            let Envelope { id, data: req } = match req_env.clone().generate_envelope() {
                Ok(env) => env,
                Err(e) => {
                    return Response::new(500).body_string(e);
                }
            };

            // Call into libqaul via the rpc utilities
            let responder: Arc<_> = Arc::clone(r.state());
            let resp = responder.respond(req).await;

            let env = Envelope { id, data: resp };

            // Build the reply envelope
            let resp_env: ResponseEnv = (env, req_env).into();
            Response::new(200).body_json(&resp_env).unwrap()
        }
    });

    app_rpc
}
