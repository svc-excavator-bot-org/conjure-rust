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

//! The Conjure HTTP client API.

use async_trait::async_trait;
use bytes::Bytes;
use conjure_error::Error;
use futures_core::future::BoxFuture;
use futures_core::Stream;
use http::{Request, Response};
use std::io::Write;
use std::pin::Pin;

/// A trait implemented by generated blocking client interfaces for a Conjure service.
pub trait Service<C> {
    /// Creates a new service wrapping an HTTP client.
    fn new(client: C) -> Self;
}

/// A trait implemented by generated async client interfaces for a Conjure service.
pub trait AsyncService<C> {
    /// Creates a new service wrapping an async HTTP client.
    fn new(client: C) -> Self;
}

/// Conjure-specific metadata about an endpoint.
///
/// This is included as an extension in all `Request`s passed to blocking and async Conjure clients.
pub struct Endpoint {
    service: &'static str,
    version: Option<&'static str>,
    name: &'static str,
}

impl Endpoint {
    /// Creates a new `Endpoint`.
    #[inline]
    pub fn new(service: &'static str, version: Option<&'static str>, name: &'static str) -> Self {
        Endpoint {
            service,
            version,
            name,
        }
    }

    /// Returns the name of the service the endpoint is part of.
    #[inline]
    pub fn service(&self) -> &'static str {
        self.service
    }

    /// Returns the version of the Conjure definition defining the service, if known.
    #[inline]
    pub fn version(&self) -> Option<&'static str> {
        self.version
    }

    /// Returns the name of the endpoint.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// The body of a Conjure request.
pub enum Body<T> {
    /// No body.
    Empty,
    /// A body already buffered in memory.
    Fixed(Bytes),
    /// A streaming body.
    Streaming(T),
}

/// The body of a blocking Conjure request.
pub type BlockingBody<'a, W> = Body<&'a mut dyn WriteBody<W>>;

/// The body of an async Conjure request.
pub type AsyncBody<'a, W> = Body<Pin<&'a mut (dyn AsyncWriteBody<W> + Send)>>;

/// A trait implemented by HTTP client implementations.
pub trait Client {
    /// The client's binary request write type.
    type BodyWriter;
    /// The client's binary response body type.
    type ResponseBody: Iterator<Item = Result<Bytes, Error>>;

    /// Makes an HTTP request.
    ///
    /// The request's URI will be in absolute-form and it will always contain an `Endpoint` object in its extensions.
    ///
    /// A response must only be returned if it has a 2xx status code. The client is responsible for handling all other
    /// status codes (for example, converting a 5xx response into a service error). The client is also responsible for
    /// decoding the response body if necessary.
    fn send(
        &self,
        req: Request<BlockingBody<'_, Self::BodyWriter>>,
    ) -> Result<Response<Self::ResponseBody>, Error>;
}

/// A trait implemented by async HTTP client implementations.
pub trait AsyncClient {
    /// The client's binary request body write type.
    type BodyWriter;
    /// The client's binary response body type.
    type ResponseBody: Stream<Item = Result<Bytes, Error>>;

    /// Makes an HTTP request.
    ///
    /// The client is responsible for assembling the request URI. It is provided with the path template, unencoded path
    /// parameters, unencoded query parameters, header parameters, and request body.
    ///
    /// A response must only be returned if it has a 2xx status code. The client is responsible for handling all other
    /// status codes (for example, converting a 5xx response into a service error). The client is also responsible for
    /// decoding the response body if necessary.
    fn send<'a>(
        &'a self,
        req: Request<AsyncBody<'a, Self::BodyWriter>>,
    ) -> BoxFuture<'a, Result<Response<Self::ResponseBody>, Error>>;
}

/// A trait implemented by streaming bodies.
pub trait WriteBody<W> {
    /// Writes the body out, in its entirety.
    ///
    /// Behavior is unspecified if this method is called twice without a successful call to `reset` in between.
    fn write_body(&mut self, w: &mut W) -> Result<(), Error>;

    /// Attempts to reset the body so that it can be written out again.
    ///
    /// Returns `true` if successful. Behavior is unspecified if this is not called after a call to `write_body`.
    fn reset(&mut self) -> bool;
}

impl<W> WriteBody<W> for &[u8]
where
    W: Write,
{
    fn write_body(&mut self, w: &mut W) -> Result<(), Error> {
        w.write_all(self).map_err(Error::internal_safe)
    }

    fn reset(&mut self) -> bool {
        true
    }
}

/// A trait implemented by async streaming bodies.
///
/// This trait can most easily be implemented with the [async-trait crate](https://docs.rs/async-trait).
///
/// # Examples
///
/// ```ignore
/// use async_trait::async_trait;
/// use conjure_error::Error;
/// use conjure_http::client::AsyncWriteBody;
/// use std::pin::Pin;
/// use tokio_io::{AsyncWrite, AsyncWriteExt};
///
/// pub struct SimpleBodyWriter;
///
/// #[async_trait]
/// impl<W> AsyncWriteBody<W> for SimpleBodyWriter
/// where
///     W: AsyncWrite + Send,
/// {
///     async fn write_body(self: Pin<&mut Self>, mut w: Pin<&mut W>) -> Result<(), Error> {
///         w.write_all(b"hello world").await.map_err(Error::internal_safe)
///     }
///
///     async fn reset(self: Pin<&mut Self>) -> bool
///     where
///         W: 'async_trait,
///     {
///         true
///     }
/// }
/// ```
#[async_trait]
pub trait AsyncWriteBody<W> {
    /// Writes the body out, in its entirety.
    ///
    /// Behavior is unspecified if this method is called twice without a successful call to `reset` in between.
    async fn write_body(self: Pin<&mut Self>, w: Pin<&mut W>) -> Result<(), Error>;

    /// Attempts to reset the body so that it can be written out again.
    ///
    /// Returns `true` if successful. Behavior is unspecified if this is not called after a call to `write_body`.
    async fn reset(self: Pin<&mut Self>) -> bool
    where
        W: 'async_trait;
}
