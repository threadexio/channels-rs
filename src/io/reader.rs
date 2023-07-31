use core::any::type_name;
use core::fmt;

#[cfg(feature = "statistics")]
use crate::stats;

pub struct Reader<R> {
	inner: R,

	#[cfg(feature = "statistics")]
	pub stats: stats::RecvStats,
}

impl<R> Reader<R> {
	pub fn new(reader: R) -> Self {
		Self {
			inner: reader,

			#[cfg(feature = "statistics")]
			stats: stats::RecvStats::new(),
		}
	}

	pub fn get(&self) -> &R {
		&self.inner
	}

	pub fn get_mut(&mut self) -> &mut R {
		&mut self.inner
	}
}

impl<R> fmt::Debug for Reader<R> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_struct("Reader");
		s.field("inner", &type_name::<R>());

		#[cfg(feature = "statistics")]
		s.field("stats", &self.stats);

		s.finish()
	}
}

mod sync_impl {
	use super::*;

	use std::io::{Read, Result};

	impl<R> Read for Reader<R>
	where
		R: Read,
	{
		fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
			let i = self.inner.read(buf)?;

			#[cfg(feature = "statistics")]
			self.stats.add_received(i);

			Ok(i)
		}
	}
}

#[cfg(feature = "tokio")]
mod async_tokio_impl {
	use super::*;

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
}
