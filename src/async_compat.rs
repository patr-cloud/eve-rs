use std::pin::Pin;
use std::task::{Context, Poll};

use async_std::io;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;

#[derive(Clone)]
pub struct HyperExecutor;

impl<F> hyper::rt::Executor<F> for HyperExecutor
where
	F: Future + Send + 'static,
	F::Output: Send + 'static,
{
	fn execute(&self, fut: F) {
		task::spawn(fut);
	}
}

pub struct HyperListener(pub TcpListener);

impl hyper::server::accept::Accept for HyperListener {
	type Conn = HyperStream;
	type Error = io::Error;

	fn poll_accept(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
		let stream = task::ready!(Pin::new(&mut self.0.incoming()).poll_next(cx)).unwrap()?;
		Poll::Ready(Some(Ok(HyperStream(stream))))
	}
}

pub struct HyperStream(pub TcpStream);

impl tokio::io::AsyncRead for HyperStream {
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut [u8],
	) -> Poll<io::Result<usize>> {
		Pin::new(&mut self.0).poll_read(cx, buf)
	}
}

impl tokio::io::AsyncWrite for HyperStream {
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &[u8],
	) -> Poll<io::Result<usize>> {
		Pin::new(&mut self.0).poll_write(cx, buf)
	}

	fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
		Pin::new(&mut self.0).poll_flush(cx)
	}

	fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
		Pin::new(&mut self.0).poll_close(cx)
	}
}
