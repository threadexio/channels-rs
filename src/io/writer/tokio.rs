use super::Writer;

use core::marker::Unpin;
use core::pin::Pin;

use std::io::Result;
use std::task::{Context, Poll};

use tokio::io::AsyncWrite;

impl<W> AsyncWrite for Writer<W>
where
	W: AsyncWrite + Unpin,
{
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &[u8],
	) -> Poll<Result<usize>> {
		match Pin::new(&mut self.inner).poll_write(cx, buf) {
			Poll::Ready(Ok(n)) => {
				Poll::Ready(self.on_write(buf, n).map(|_| n))
			},
			r => r,
		}
	}

	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Result<()>> {
		Pin::new(&mut self.inner).poll_flush(cx)
	}

	fn poll_shutdown(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Result<()>> {
		Pin::new(&mut self.inner).poll_shutdown(cx)
	}
}
