mod common;

use common::{ConsulServer, ConsulServerHelper};
use consulrs::{api::kv::common::KVPair, api::kv::requests, client::Client, kv};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use test_log::test;

#[derive(Deserialize, Serialize)]
struct TestObject {
    pub field: String,
}

#[test]
fn test() {
    let test = common::new_test();
    test.run(|instance| async move {
        let server: ConsulServer = instance.server();
        let client = server.client();
        let key = "test";

        test_set(&client, key).await;
        test_keys(&client).await;
        test_read(&client, key).await;
        test_read_recurse(&client, key).await;
        test_read_raw(&client, key).await;
        test_delete(&client, key).await;
        test_json(&client, key).await;
        test_roundtrip_bytes(&client, key).await;
    });
}

async fn test_delete(client: &impl Client, key: &str) {
    let res = kv::delete(client, key, None).await;
    assert!(res.is_ok());
}

async fn test_json(client: &impl Client, key: &str) {
    let obj = TestObject {
        field: "test".into(),
    };

    let res = kv::set_json(client, key, &obj, None).await;
    assert!(res.is_ok());

    let res = kv::read_json::<TestObject, _>(client, key, None).await;
    assert!(res.is_ok());

    assert_eq!(obj.field, res.unwrap().response.value.field);
}

async fn test_keys(client: &impl Client) {
    let res = kv::keys(client, "", None).await;
    assert!(res.is_ok());
}

async fn test_read_raw(client: &impl Client, key: &str) {
    let res = kv::read_raw(client, key, None).await;
    assert!(res.is_ok());
}

async fn test_read(client: &impl Client, key: &str) {
    let res = kv::read(client, key, None).await;
    assert!(res.is_ok());
}

async fn test_set(client: &impl Client, key: &str) {
    let res = kv::set(client, key, b"test", None).await;
    assert!(res.is_ok());
}

fn read_response_to_value(mut response: Vec<KVPair>) -> Vec<u8> {
    response
        .pop()
        .and_then(|v| v.value)
        .unwrap()
        .try_into()
        .unwrap()
}

async fn test_roundtrip_bytes(client: &impl Client, key: &str) {
    let res = kv::set(client, key, b"test", None).await;
    assert!(res.is_ok());

    let res = kv::read(client, key, None).await;
    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res.response.len(), 1);

    let bytes: Vec<u8> = read_response_to_value(res.response);
    assert_eq!(bytes, b"test");
}

async fn test_read_recurse(client: &impl Client, key: &str) {
    let res = kv::read(
        client,
        key,
        Some(requests::ReadKeyRequestBuilder::default().recurse(true)),
    )
    .await;
    assert!(res.is_ok());
}
