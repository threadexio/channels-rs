use core::any::type_name;
use core::fmt;

#[cfg(feature = "statistics")]
use crate::stats;

pub struct Writer<W> {
	inner: W,

	#[cfg(feature = "statistics")]
	pub stats: stats::SendStats,
}

impl<W> Writer<W> {
	pub fn new(writer: W) -> Self {
		Self {
			inner: writer,

			#[cfg(feature = "statistics")]
			stats: stats::SendStats::new(),
		}
	}

	pub fn get(&self) -> &W {
		&self.inner
	}

	pub fn get_mut(&mut self) -> &mut W {
		&mut self.inner
	}
}

impl<W> fmt::Debug for Writer<W> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_struct("Writer");
		s.field("inner", &type_name::<W>());

		#[cfg(feature = "statistics")]
		s.field("stats", &self.stats);

		s.finish()
	}
}

mod sync_impl {
	use super::*;

	use std::io::{Result, Write};

	impl<W> Write for Writer<W>
	where
		W: Write,
	{
		fn write(&mut self, buf: &[u8]) -> Result<usize> {
			let i = self.inner.write(buf)?;

			#[cfg(feature = "statistics")]
			self.stats.add_sent(i);

			Ok(i)
		}

		fn flush(&mut self) -> Result<()> {
			self.inner.flush()
		}
	}
}

#[cfg(feature = "tokio")]
mod async_tokio_impl {
	use super::*;

	use std::io::Result;
	use std::marker::Unpin;
	use std::pin::Pin;
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
				Poll::Ready(Ok(i)) => {
					#[cfg(feature = "statistics")]
					self.stats.add_sent(i);

					Poll::Ready(Ok(i))
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
}
