//! # HTTP-API to RPC transformation


use crate::Responder;
use async_std::sync::Arc;
use libqaul_rpc1::{
    json::{JsonAuth, JsonMap, RequestEnv},
    Envelope,
};
use mime::APPLICATION_JSON;
use serde_json::json;
use std::collections::BTreeMap;
use tide::{self, Request, Response};

/// Convert the http-api call to RPC
pub async fn http2rpc(r: Request<Arc<Responder>>, kind: &str, method: &str) -> Response {
    http2rpc_params_query(r, kind, method, None, false).await
}

/// Convert the http-api call to RPC convert the query string to params
pub async fn http2rpc_query(r: Request<Arc<Responder>>, kind: &str, method: &str) -> Response {
    http2rpc_params_query(r, kind, method, None, true).await
}

/// Convert the http-api call to RPC with inserting the URI paramters 
/// into the data BTree
pub async fn http2rpc_params(
    mut r: Request<Arc<Responder>>,
    kind: &str,
    method: &str,
    uri_params: Option<Vec<&str>>,
) -> Response {
    http2rpc_params_query(r, kind, method, uri_params, false).await
}

/// Convert the http-api call to RPC with inserting the URI paramters 
/// and query parameters into the data BTree
pub async fn http2rpc_params_query(
    mut r: Request<Arc<Responder>>,
    kind: &str,
    method: &str,
    uri_params: Option<Vec<&str>>,
    uri_query: bool,
) -> Response {
    // display debugging information about the 
    println!("http2rpc_params {} {}", kind, method);

    // get Authorization from header
    let auth: Option<JsonAuth> = match r.header("Authorization") {
        None => None,
        Some(s) => {
            if let Ok(json_auth) = serde_json::from_str(s) {
                Some(json_auth)
            } else {
                return Response::new(400)
                    .body_string(
                        "{\"Error\":\"Malformed json in authentication header\"}".to_string(),
                    )
                    .set_mime(APPLICATION_JSON);
            }
        }
    };

    // get request body for certain methods
    // TODO: should return an HTTP error: when there is no body on POST or PATCH
    let mut data: JsonMap = match r.body_string().await {
        Ok(o) => match serde_json::from_str(o.as_str()) {
            Ok(o2) => o2,
            Err(_) => BTreeMap::new(),
        },
        Err(_) => BTreeMap::new(),
    };

    // // change data to diff if it is a PATCH request
    // if method == "modify" {
    //     for (key, value) in data.iter_mut() {
    //         if key != "id" {
    //             *value = json!({ "set": value });
    //         }
    //     }
    // }

    // debug information
    println!("--------- data ---------");
    println!("{:?}", data.clone());
    println!("--------- end data ---------");

    // get values from URI parameters
    if let Some(params) = uri_params {
        params.iter().for_each(|v| {
            if let Ok(x) = r.param(v) {
                data.insert(v.to_string(), serde_json::Value::String(x));
            } else {
                // return Response::new(400)
                //     .body_string("{\"Error\":\"URI parameter parsing error.\"}".to_string())
                //     .set_mime(APPLICATION_JSON);
            }
        });
    }

    // query parameters for query/list requests
    if uri_query {
        if let Some(query) = r.uri().query() {
            query.split("&").collect::<Vec<&str>>().iter().for_each(|tuple| {
                let q_item = tuple.split("=").collect::<Vec<&str>>();
                if q_item.len() == 2 {
                    data.insert(q_item[0].to_string(), serde_json::Value::String(q_item[1].to_string()));
                }
            });
        }
    };

    // create RPC structure
    let rpc_req = RequestEnv {
        id: String::from(r.uri().path()),
        kind: kind.to_string(),
        method: method.to_string(),
        page: None,
        auth,
        data,
    };

    let Envelope { id: _, data: req } = match rpc_req.clone().generate_envelope() {
        Ok(env) => env,
        Err(e) => {
            // If there was an error parsing the envelope, return it
            return Response::new(500).body_string(e);
        }
    };

    // Call into libqaul via the rpc utilities
    let responder: Arc<_> = Arc::clone(r.state());
    let resp = responder.respond(req).await;

    // Return success or error code values
    let return_code = match &resp {
        libqaul_rpc1::Response::Error(s) => {
            if s.starts_with("Not authorised") {
                401
            } else {
                400
            }
        }
        libqaul_rpc1::Response::Success => {
            // return empty body when there is only the generic Success message
            return Response::new(204).body_string("".to_string());
        }
        _ => 200,
    };

    Response::new(return_code).body_json(&resp).unwrap()
}
