// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite, BufWriter};
use tokio_io_timeout::TimeoutStream;

pub trait SessionStream: AsyncRead + AsyncWrite + Unpin + Send + Sync + std::fmt::Debug {
    //  Change the read timeout on the session stream.
    // fn set_read_timeout(&mut self, timeout: Option<Duration>);
}

impl SessionStream for Box<dyn SessionStream> {
    // fn set_read_timeout(&mut self, timeout: Option<Duration>) {
    //     self.as_mut().set_read_timeout(timeout);
    // }
}

impl<T: SessionStream> SessionStream for tokio_rustls::client::TlsStream<T> {
    // fn set_read_timeout(&mut self, timeout: Option<Duration>) {
    //     self.get_mut().0.set_read_timeout(timeout);
    // }
}

impl<T: SessionStream> SessionStream for BufWriter<T> {
    // fn set_read_timeout(&mut self, timeout: Option<Duration>) {
    //     self.get_mut().set_read_timeout(timeout);
    // }
}
impl<T: AsyncRead + AsyncWrite + Send + Sync + std::fmt::Debug> SessionStream
    for Pin<Box<TimeoutStream<T>>>
{
    // fn set_read_timeout(&mut self, timeout: Option<Duration>) {
    //     self.as_mut().set_read_timeout_pinned(timeout);
    // }
}
