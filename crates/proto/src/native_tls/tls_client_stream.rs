// Copyright 2015-2016 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// https://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! TlsClientStream for DNS over TLS

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

use futures_util::TryFutureExt;
use native_tls::Certificate;
use tokio_native_tls::TlsStream as TokioTlsStream;

use crate::error::ProtoError;
use crate::native_tls::TlsStreamBuilder;
use crate::runtime::iocompat::{AsyncIoStdAsTokio, AsyncIoTokioAsStd};
use crate::runtime::RuntimeProvider;
use crate::tcp::TcpClientStream;
use crate::xfer::BufDnsStreamHandle;

/// TlsClientStream secure DNS over TCP stream
///
/// See TlsClientStreamBuilder::new()
pub type TlsClientStream<S> =
    TcpClientStream<AsyncIoTokioAsStd<TokioTlsStream<AsyncIoStdAsTokio<S>>>>;

/// Builder for TlsClientStream
pub struct TlsClientStreamBuilder<P>(TlsStreamBuilder<P>);

impl<P: RuntimeProvider> TlsClientStreamBuilder<P> {
    /// Creates a builder fo the construction of a TlsClientStream
    pub fn new(provider: P) -> Self {
        Self(TlsStreamBuilder::new(provider))
    }

    /// Add a custom trusted peer certificate or certificate authority.
    ///
    /// If this is the 'client' then the 'server' must have it associated as it's `identity`, or have had the `identity` signed by this certificate.
    pub fn add_ca(&mut self, ca: Certificate) {
        self.0.add_ca(ca);
    }

    /// Sets the address to connect from.
    pub fn bind_addr(&mut self, bind_addr: SocketAddr) {
        self.0.bind_addr(bind_addr);
    }

    /// Creates a new TlsStream to the specified name_server with stream future.
    ///
    /// # Arguments
    ///
    /// * 'future` - future of TCP stream
    /// * `name_server` - IP and Port for the remote DNS resolver
    /// * `dns_name` - The DNS name associated with a certificate
    #[allow(clippy::type_complexity)]
    pub fn build_with_future<F>(
        self,
        future: F,
        name_server: SocketAddr,
        dns_name: String,
    ) -> (
        Pin<Box<dyn Future<Output = Result<TlsClientStream<P::Tcp>, ProtoError>> + Send>>,
        BufDnsStreamHandle,
    )
    where
        F: Future<Output = std::io::Result<P::Tcp>> + Send + Unpin + 'static,
    {
        let (stream_future, sender) = self.0.build_with_future(future, name_server, dns_name);

        let new_future = Box::pin(
            stream_future
                .map_ok(TcpClientStream::from_stream)
                .map_err(ProtoError::from),
        );

        (new_future, sender)
    }

    /// Creates a new TlsStream to the specified name_server
    ///
    /// # Arguments
    ///
    /// * `name_server` - IP and Port for the remote DNS resolver
    /// * `dns_name` - The DNS name associated with a certificate
    #[allow(clippy::type_complexity)]
    pub fn build(
        self,
        name_server: SocketAddr,
        dns_name: String,
    ) -> (
        Pin<Box<dyn Future<Output = Result<TlsClientStream<P::Tcp>, ProtoError>> + Send>>,
        BufDnsStreamHandle,
    ) {
        let (stream_future, sender) = self.0.build(name_server, dns_name);

        let new_future = Box::pin(
            stream_future
                .map_ok(TcpClientStream::from_stream)
                .map_err(ProtoError::from),
        );

        (new_future, sender)
    }
}
