//! RPC chat message tests

#[cfg(test)]
mod harness;
use harness::rpc_harness::RPC;
use std::time::Duration;

// chat_message send
#[async_std::test]
async fn rpc_chat_message_create() {
    // create RPC environment
    let rpc = RPC::init().await;

    // create a user on each node
    let user_a = rpc.network.a().users().create("123456").await.unwrap();
    let user_b = rpc.network.b().users().create("123456").await.unwrap();

    // create a chat room
    let room = rpc
        .responder_a
        .chat
        .start_chat(user_a.clone(), vec![user_b.0], None)
        .await
        .unwrap();

    // Send Message from user A
    // RPC JSON input
    let json_string = format!(
        r#"{{
        "id": "/chat_message/create",
        "kind": "chat_message",
        "method": "create",
        "data": {{
            "text": "hello world!",
            "room": "{room_id}"
        }},
        "auth": {{
            "id": "{a_id}",
            "token": "{a_token}"
        }}
    }}"#,
        room_id = room.id,
        a_id = user_a.0,
        a_token = user_a.1
    );

    // send JSON
    let resp = rpc.send_a(json_string.as_str()).await;

    // check result
    dbg!(resp.clone());
    assert!(resp.data.contains_key("chat_message"));
    assert_eq!(resp.data.get("chat_message").unwrap().get("content").unwrap(), "hello world!");
}

// chat_message receive
#[async_std::test]
async fn rpc_chat_message_list() {
    // create RPC environment
    let rpc = RPC::init().await;

    // create a user on each node
    let user_a = rpc.network.a().users().create("123456").await.unwrap();
    let user_b = rpc.network.b().users().create("123456").await.unwrap();

    // create a chat room
    let room = rpc
        .responder_a
        .chat
        .start_chat(user_a.clone(), vec![user_b.0], None)
        .await
        .unwrap();

    // Send Message from user A
    let _msg = rpc
        .responder_a
        .chat
        .send_message(user_a.clone(), room.id, "hello world!".to_string())
        .await
        .unwrap();

    // wait until message is delivered
    async_std::task::sleep(Duration::from_secs(1)).await;

    // Receive Message at user B
    // RPC JSON input
    let json_string = format!(
        r#"{{
        "id": "/chat_message/query",
        "kind": "chat_message",
        "method": "query",
        "data": {{
            "chat_room": "{room_id}"
        }},
        "auth": {{
            "id": "{b_id}",
            "token": "{b_token}"
        }}
    }}"#,
        room_id = room.id,
        b_id = user_b.0,
        b_token = user_b.1
    );

    // send JSON
    let resp = rpc.send_b(json_string.as_str()).await;

    // check result
    dbg!(resp.clone());
    assert!(resp.data.contains_key("chat_messages"));
    assert!(resp.data["chat_messages"].as_array().unwrap().len() > 1);
    assert_eq!(resp.data.get("chat_messages").unwrap().get(1).unwrap().get("content").unwrap(), "hello world!");
}
