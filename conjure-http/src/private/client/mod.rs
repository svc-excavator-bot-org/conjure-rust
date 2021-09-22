// Copyright 2019 Palantir Technologies, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use crate::client::Body;
pub use crate::private::client::uri_builder::UriBuilder;
use bytes::{Bytes, BytesMut};
use conjure_error::Error;
use conjure_object::{BearerToken, Plain, ToPlain};
use conjure_serde::json;
use futures_core::Stream;
use futures_util::TryStreamExt;
use http::header::{
    HeaderName, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, COOKIE,
};
use http::{Request, Response, StatusCode};
use once_cell::sync::Lazy;
use pin_utils::pin_mut;
use serde::de::{DeserializeOwned, IgnoredAny};
use serde::Serialize;

mod uri_builder;

static APPLICATION_JSON: Lazy<HeaderValue> =
    Lazy::new(|| HeaderValue::from_static("application/json"));
static APPLICATION_OCTET_STREAM: Lazy<HeaderValue> =
    Lazy::new(|| HeaderValue::from_static("application/octet-stream"));

pub fn encode_empty_request<S>() -> Request<Body<S>> {
    Request::new(Body::Empty)
}

pub fn encode_serializable_request<T, S>(body: &T) -> Request<Body<S>>
where
    T: Serialize,
{
    let buf = json::to_vec(body).unwrap();
    let len = buf.len();

    let mut request = Request::new(Body::Fixed(Bytes::from(buf)));
    request
        .headers_mut()
        .insert(CONTENT_TYPE, APPLICATION_JSON.clone());
    request
        .headers_mut()
        .insert(CONTENT_LENGTH, HeaderValue::from(len));

    request
}

pub fn encode_binary_request<S>(body: S) -> Request<Body<S>> {
    let mut request = Request::new(Body::Streaming(body));
    request
        .headers_mut()
        .insert(CONTENT_TYPE, APPLICATION_OCTET_STREAM.clone());

    request
}

pub fn encode_empty_response_headers<B>(request: &mut Request<B>) {
    encode_serializable_response_headers(request);
}

pub fn encode_serializable_response_headers<B>(request: &mut Request<B>) {
    request
        .headers_mut()
        .insert(ACCEPT, APPLICATION_JSON.clone());
}

pub fn encode_binary_response_headers<B>(request: &mut Request<B>) {
    request
        .headers_mut()
        .insert(ACCEPT, APPLICATION_OCTET_STREAM.clone());
}

pub fn encode_header<B>(
    request: &mut Request<B>,
    header: &'static str,
    value: &dyn Plain,
) -> Result<(), Error> {
    let header = HeaderName::from_static(header);
    let value = HeaderValue::from_maybe_shared(Bytes::from(value.to_plain()))
        .map_err(Error::internal_safe)?;
    request.headers_mut().insert(header, value);

    Ok(())
}

pub fn encode_optional_header<B, T>(
    request: &mut Request<B>,
    header: &'static str,
    value: &Option<T>,
) -> Result<(), Error>
where
    T: Plain,
{
    if let Some(value) = value {
        encode_header(request, header, value)?;
    }

    Ok(())
}

pub fn encode_cookie_auth<B>(request: &mut Request<B>, prefix: &str, value: &BearerToken) {
    encode_auth(request, COOKIE, prefix, value)
}

pub fn encode_header_auth<B>(request: &mut Request<B>, value: &BearerToken) {
    encode_auth(request, AUTHORIZATION, "Bearer ", value);
}

fn encode_auth<B>(request: &mut Request<B>, header: HeaderName, prefix: &str, value: &BearerToken) {
    let value = format!("{}{}", prefix, value.as_str());
    let value = HeaderValue::from_maybe_shared(Bytes::from(value))
        .expect("bearer tokens are valid headers");
    request.headers_mut().insert(header, value);
}

// The logic here is unfortunately pretty much duplicated between blocking and async, but there isn't really a way to
// combine them :(.

pub fn decode_empty_response<I>(response: Response<I>) -> Result<(), Error>
where
    I: Iterator<Item = Result<Bytes, Error>>,
{
    if response.status() == StatusCode::NO_CONTENT {
        return Ok(());
    }

    // Servers can send a JSON response to an endpoint we expect to be void. Rather than just ignoring the response
    // body, we're going to "deserialize" it to IgnoredAny to validate that it is in fact a valid JSON body and to
    // consume the response body data so the socket can be reused for another request.
    decode_serializable_response::<IgnoredAny, _>(response)?;

    Ok(())
}

pub async fn async_decode_empty_response<I>(response: Response<I>) -> Result<(), Error>
where
    I: Stream<Item = Result<Bytes, Error>>,
{
    if response.status() == StatusCode::NO_CONTENT {
        return Ok(());
    }

    async_decode_serializable_response::<IgnoredAny, _>(response).await?;

    Ok(())
}

pub fn decode_default_serializable_response<T, I>(response: Response<I>) -> Result<T, Error>
where
    T: DeserializeOwned + Default,
    I: Iterator<Item = Result<Bytes, Error>>,
{
    if response.status() == StatusCode::NO_CONTENT {
        return Ok(T::default());
    }

    decode_serializable_response(response)
}

pub async fn async_decode_default_serializable_response<T, I>(
    response: Response<I>,
) -> Result<T, Error>
where
    T: DeserializeOwned + Default,
    I: Stream<Item = Result<Bytes, Error>>,
{
    if response.status() == StatusCode::NO_CONTENT {
        return Ok(T::default());
    }

    async_decode_serializable_response(response).await
}

pub fn decode_serializable_response<T, I>(response: Response<I>) -> Result<T, Error>
where
    T: DeserializeOwned,
    I: Iterator<Item = Result<Bytes, Error>>,
{
    if response.headers().get(CONTENT_TYPE) != Some(&*APPLICATION_JSON) {
        return Err(Error::internal_safe("invalid response Content-Type"));
    }

    let body = read_body(response.into_body())?;
    let body = json::client_from_slice(&body).map_err(Error::internal)?;

    Ok(body)
}

pub async fn async_decode_serializable_response<T, I>(response: Response<I>) -> Result<T, Error>
where
    T: DeserializeOwned,
    I: Stream<Item = Result<Bytes, Error>>,
{
    if response.headers().get(CONTENT_TYPE) != Some(&*APPLICATION_JSON) {
        return Err(Error::internal("invalid response Content-Type"));
    }

    let body = async_read_body(response.into_body()).await?;
    let body = json::client_from_slice(&body).map_err(Error::internal)?;

    Ok(body)
}

pub fn decode_optional_binary_response<I>(response: Response<I>) -> Result<Option<I>, Error> {
    if response.status() == StatusCode::NO_CONTENT {
        return Ok(None);
    }

    decode_binary_response(response).map(Some)
}

pub fn decode_binary_response<I>(response: Response<I>) -> Result<I, Error> {
    if response.headers().get(CONTENT_TYPE) != Some(&*APPLICATION_OCTET_STREAM) {
        return Err(Error::internal_safe("invalid response Content-Type"));
    }

    Ok(response.into_body())
}

// slightly nontrivial to avoid a copy for single-chunk responses
fn read_body<I>(mut body: I) -> Result<Bytes, Error>
where
    I: Iterator<Item = Result<Bytes, Error>>,
{
    let first = match body.next().transpose()? {
        Some(bytes) => bytes,
        None => return Ok(Bytes::new()),
    };

    let mut buf = BytesMut::new();
    match body.next().transpose()? {
        Some(second) => {
            buf.reserve(first.len() + second.len());
            buf.extend_from_slice(&first);
            buf.extend_from_slice(&second);
        }
        None => return Ok(first),
    };

    for bytes in body {
        buf.extend_from_slice(&bytes?);
    }

    Ok(buf.freeze())
}

async fn async_read_body<I>(body: I) -> Result<Bytes, Error>
where
    I: Stream<Item = Result<Bytes, Error>>,
{
    pin_mut!(body);

    let first = match body.try_next().await? {
        Some(bytes) => bytes,
        None => return Ok(Bytes::new()),
    };

    let mut buf = BytesMut::new();
    match body.try_next().await? {
        Some(second) => {
            buf.reserve(first.len() + second.len());
            buf.extend_from_slice(&first);
            buf.extend_from_slice(&second);
        }
        None => return Ok(first),
    }

    while let Some(bytes) = body.try_next().await? {
        buf.extend_from_slice(&bytes);
    }

    Ok(buf.freeze())
}
