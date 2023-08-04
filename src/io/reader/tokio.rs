use super::Reader;

use core::marker::Unpin;
use core::pin::Pin;

use std::io::Result;
use std::task::{ready, Context, Poll};

use tokio::io::{AsyncRead, ReadBuf};

impl<R> AsyncRead for Reader<R>
where
	R: AsyncRead + Unpin,
{
	#[allow(unused_variables)]
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
		buf: &mut ReadBuf<'_>,
	) -> Poll<Result<()>> {
		let start = buf.filled().len();

		let result =
			ready!(Pin::new(&mut self.inner).poll_read(cx, buf));

		let end = buf.filled().len();

		// Unfortunately the `poll_read` method does not actually
		// return the number of bytes it read. So this means we must
		// calculate the delta to find out how many bytes it read.
		let delta_bytes = end - start;

		#[cfg(feature = "statistics")]
		self.stats.add_received(delta_bytes);

		Poll::Ready(result)
	}
}
