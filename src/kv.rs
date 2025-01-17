use std::convert::TryInto;

use crate::{
    api::{
        self,
        kv::{
            common::{GenericKVPair, KVPair},
            requests::{
                DeleteKeyRequest, DeleteKeyRequestBuilder, ReadKeyRequest, ReadKeyRequestBuilder,
                ReadKeysRequest, ReadKeysRequestBuilder, ReadRawKeyRequest,
                ReadRawKeyRequestBuilder, SetKeyRequest, SetKeyRequestBuilder,
            },
        },
        ApiResponse,
    },
    client::Client,
    error::ClientError,
};
use serde::{de::DeserializeOwned, Serialize};

/// Deletes the given key.
///
/// See [DeleteKeyRequest]
#[instrument(skip(client, opts), err)]
pub async fn delete(
    client: &impl Client,
    key: &str,
    opts: Option<&mut DeleteKeyRequestBuilder>,
) -> Result<ApiResponse<bool>, ClientError> {
    let mut t = DeleteKeyRequest::builder();
    let endpoint = opts.unwrap_or(&mut t).key(key).build().unwrap();
    api::exec_with_result(client, endpoint).await
}

/// Lists all keys at the given path.
///
/// See [ReadKeysRequest]
#[instrument(skip(client, opts), err)]
pub async fn keys(
    client: &impl Client,
    path: &str,
    opts: Option<&mut ReadKeysRequestBuilder>,
) -> Result<ApiResponse<Vec<String>>, ClientError> {
    let mut t = ReadKeysRequest::builder();
    let endpoint = opts.unwrap_or(&mut t).key(path).build().unwrap();
    api::exec_with_result(client, endpoint).await
}

/// Reads the raw value at the given key.
///
/// See [ReadKeyRequest]
#[instrument(skip(client, opts), err)]
pub async fn read_raw(
    client: &impl Client,
    key: &str,
    opts: Option<&mut ReadRawKeyRequestBuilder>,
) -> Result<ApiResponse<Vec<u8>>, ClientError> {
    let mut t = ReadRawKeyRequest::builder();
    let endpoint = opts.unwrap_or(&mut t).key(key).build().unwrap();
    api::exec_with_raw(client, endpoint).await
}

/// Reads the value at the given key.
///
/// See [ReadKeyRequest]
#[instrument(skip(client, opts), err)]
pub async fn read(
    client: &impl Client,
    key: &str,
    opts: Option<&mut ReadKeyRequestBuilder>,
) -> Result<ApiResponse<Vec<KVPair>>, ClientError> {
    let mut t = ReadKeyRequest::builder();
    let endpoint = opts.unwrap_or(&mut t).key(key).build().unwrap();
    api::exec_with_result(client, endpoint).await
}

/// Reads the JSON value at the given key and deserializes it into an object.
///
/// If the API call returns an empty list then this function will return a
/// [ClientError::EmptyResponseError]. Note that the function only handles a
/// single value - only the first element in the list is parsed and returned.
///
/// See [ReadKeyRequest]
#[instrument(skip(client, opts), err)]
pub async fn read_json<T: DeserializeOwned, C: Client>(
    client: &C,
    key: &str,
    opts: Option<&mut ReadKeyRequestBuilder>,
) -> Result<ApiResponse<GenericKVPair<T>>, ClientError> {
    let mut t = ReadKeyRequest::builder();
    let endpoint = opts.unwrap_or(&mut t).key(key).build().unwrap();
    let mut res = api::exec_with_result(client, endpoint).await?;

    if !res.response.is_empty() {
        let kv = res.response.pop().unwrap();
        let bytes: Vec<u8> = kv.value.unwrap().try_into()?;
        let t = serde_json::from_slice(&bytes)
            .map_err(|e| ClientError::JsonDeserializeError { source: e })?;
        let gkv = GenericKVPair {
            value: t,
            create_index: kv.create_index,
            flags: kv.flags,
            key: kv.key,
            lock_index: kv.lock_index,
            modify_index: kv.modify_index,
            namespace: kv.namespace,
            session: kv.session,
        };
        Ok(ApiResponse {
            response: gkv,
            cache: res.cache,
            content_hash: res.content_hash,
            default_acl_policy: res.default_acl_policy,
            index: res.index,
            known_leader: res.known_leader,
            last_contact: res.last_contact,
            query_backend: res.query_backend,
        })
    } else {
        Err(ClientError::EmptyResponseError)
    }
}

/// Reads the raw JSON value at the given key and deserializes it into an object.
///
/// See [ReadRawKeyRequest]
#[instrument(skip(client, opts), err)]
pub async fn read_json_raw<T: DeserializeOwned, C: Client>(
    client: &C,
    key: &str,
    opts: Option<&mut ReadRawKeyRequestBuilder>,
) -> Result<ApiResponse<T>, ClientError> {
    let mut t = ReadRawKeyRequest::builder();
    let endpoint = opts.unwrap_or(&mut t).key(key).build().unwrap();
    let res = api::exec_with_raw(client, endpoint).await?;

    if !res.response.is_empty() {
        let t = serde_json::from_slice(&res.response)
            .map_err(|e| ClientError::JsonDeserializeError { source: e })?;
        Ok(ApiResponse {
            response: t,
            cache: res.cache,
            content_hash: res.content_hash,
            default_acl_policy: res.default_acl_policy,
            index: res.index,
            known_leader: res.known_leader,
            last_contact: res.last_contact,
            query_backend: res.query_backend,
        })
    } else {
        Err(ClientError::EmptyResponseError)
    }
}

/// Sets the value at the given key.
///
/// See [SetKeyRequest]
#[instrument(skip(client, value, opts), err)]
pub async fn set<'a>(
    client: &'a impl Client,
    key: &'a str,
    value: &'static [u8],
    opts: Option<&'a mut SetKeyRequestBuilder>,
) -> Result<ApiResponse<bool>, ClientError> {
    let mut t = SetKeyRequest::builder();
    let endpoint = opts
        .unwrap_or(&mut t)
        .key(key)
        .value(value)
        .build()
        .unwrap();
    api::exec_with_result(client, endpoint).await
}

/// Serializes the given value into JSON and stores it at the given key.
///
/// See [SetKeyRequest]
#[instrument(skip(client, value, opts), err)]
pub async fn set_json<T: Serialize>(
    client: &impl Client,
    key: &str,
    value: &T,
    opts: Option<&mut SetKeyRequestBuilder>,
) -> Result<ApiResponse<bool>, ClientError> {
    let mut t = SetKeyRequest::builder();
    let bytes =
        serde_json::to_vec(value).map_err(|e| ClientError::JsonSerializeError { source: e })?;
    let endpoint = opts
        .unwrap_or(&mut t)
        .key(key)
        .value(bytes)
        .build()
        .unwrap();
    api::exec_with_result(client, endpoint).await
}
