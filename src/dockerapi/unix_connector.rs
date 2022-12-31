// stolen from hyper-socket, ty hyper-socket

//! This crate provides an instance of [`Connect`] which communicates over a local Unix
//! Domain Socket rather than TCP.
//!
//! Numerous system daemons expose such sockets but use HTTP in order to unify their local and
//! remote RPC APIs (such as [Consul](https://consul.io)). This connector is a mean to communicate
//! with those services.
//!
//! NB: As sockets are named by a file path and not a DNS name, the hostname of any requests are not
//! used for initiating a connection-- all requests, regardless of the intended destination, are
//! routed to the same socket.

use futures::prelude::*;
use hyper::client::connect::{Connected, Connection};
use hyper::client::Client;
use hyper::http::Uri;
use hyper::service::Service;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf, Result};
use tokio::net::UnixStream;

/// A connector to a local Unix Domain Socket which uses HTTP as the application-layer protocol.
///
/// ```rust
/// use hyper::{Body, Client};
/// use hyper_socket::UnixSocketConnector;
///
/// let connector: UnixSocketConnector = UnixSocketConnector::new("/run/consul.sock");
/// let client: Client<_, Body> = Client::builder().build(connector);
/// ```
///
/// For more information, please refer to the [module documentation][crate].
#[derive(Clone, Debug)]
pub struct UnixSocketConnector(Arc<Path>);

impl UnixSocketConnector {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = Arc::from(path.as_ref());
        UnixSocketConnector(path)
    }

    pub fn connect(&self) -> impl Future<Output = Result<UnixSocketConnection>> {
        UnixStream::connect(Arc::clone(&self.0)).map_ok(UnixSocketConnection)
    }

    pub fn client<P: AsRef<Path>>(path: P) -> Client<Self> {
        Client::builder().build(UnixSocketConnector::new(path))
    }
}

impl Service<Uri> for UnixSocketConnector {
    type Response = UnixSocketConnection;
    type Error = tokio::io::Error;
    type Future = Pin<Box<UnixSocketFuture>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: Uri) -> Self::Future {
        Box::pin(self.connect())
    }
}

/// A wrapper around Tokio's [UnixStream][] type, implementing [Connection][].
pub struct UnixSocketConnection(UnixStream);

impl AsyncRead for UnixSocketConnection {
    #[inline(always)]
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context, buf: &mut ReadBuf) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_read(cx, buf)
    }
}

impl AsyncWrite for UnixSocketConnection {
    #[inline(always)]
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<Result<usize>> {
        Pin::new(&mut self.get_mut().0).poll_write(cx, buf)
    }

    #[inline(always)]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_flush(cx)
    }

    #[inline(always)]
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_shutdown(cx)
    }
}

impl Connection for UnixSocketConnection {
    fn connected(&self) -> Connected {
        Connected::new().proxy(true)
    }
}

impl Deref for UnixSocketConnection {
    type Target = UnixStream;

    fn deref(&self) -> &UnixStream {
        &self.0
    }
}

impl DerefMut for UnixSocketConnection {
    fn deref_mut(&mut self) -> &mut UnixStream {
        &mut self.0
    }
}

type UnixSocketFuture = dyn Future<Output = Result<UnixSocketConnection>> + Send;
