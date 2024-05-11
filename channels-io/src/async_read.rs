use core::future::Future;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use crate::{IoError, ReadBuf, ReadError};

/// This trait is the asynchronous version of [`Read`].
///
/// [`Read`]: crate::Read
pub trait AsyncRead: Unpin {
	/// Error type for [`read()`].
	///
	/// [`read()`]: AsyncRead::read
	type Error: ReadError;

	/// Asynchronously read some bytes into `buf`.
	///
	/// This function behaves in the same way as [`Read::read()`] except that it
	/// returns a [`Future`] that must be `.await`ed.
	///
	/// [`Read::read()`]: crate::Read::read
	/// [`Future`]: core::future::Future
	fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Read<'a, Self> {
		Read::new(self, buf)
	}

	/// Poll the reader and try to read some bytes into `buf`.
	///
	/// This method reads bytes into the unfilled part of `buf` and advances the
	/// it accordingly.
	fn poll_read(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut ReadBuf,
	) -> Poll<Result<(), Self::Error>> {
		default_poll_read(self, cx, buf)
	}

	/// Poll the reader once and read some bytes into the slice `buf`.
	///
	/// This method reads bytes directly into `buf` and reports how many bytes it
	/// read.
	fn poll_read_slice(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut [u8],
	) -> Poll<Result<usize, Self::Error>>;
}

fn default_poll_read<T: AsyncRead + ?Sized>(
	mut reader: Pin<&mut T>,
	cx: &mut Context,
	buf: &mut ReadBuf,
) -> Poll<Result<(), T::Error>> {
	while !buf.unfilled().is_empty() {
		match ready!(reader
			.as_mut()
			.poll_read_slice(cx, buf.unfilled_mut()))
		{
			Ok(0) => return Poll::Ready(Err(T::Error::eof())),
			Ok(n) => buf.advance(n),
			Err(e) if e.should_retry() => continue,
			Err(e) => return Poll::Ready(Err(e)),
		}
	}

	Poll::Ready(Ok(()))
}

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct Read<'a, T: ?Sized> {
	reader: &'a mut T,
	buf: ReadBuf<'a>,
}

impl<'a, T: ?Sized> Read<'a, T> {
	fn new(reader: &'a mut T, buf: &'a mut [u8]) -> Self {
		Self { reader, buf: ReadBuf::new(buf) }
	}
}

impl<'a, T> Future for Read<'a, T>
where
	T: AsyncRead + ?Sized,
{
	type Output = Result<(), T::Error>;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Self::Output> {
		let Self { ref mut reader, ref mut buf } = *self;
		Pin::new(&mut **reader).poll_read(cx, buf)
	}
}

macro_rules! forward_impl_async_read {
	($to:ty) => {
		type Error = <$to>::Error;

		fn poll_read(
			mut self: Pin<&mut Self>,
			cx: &mut Context,
			buf: &mut ReadBuf,
		) -> Poll<Result<(), Self::Error>> {
			let this = Pin::new(&mut **self);
			<$to>::poll_read(this, cx, buf)
		}

		fn poll_read_slice(
			mut self: Pin<&mut Self>,
			cx: &mut Context,
			buf: &mut [u8],
		) -> Poll<Result<usize, Self::Error>> {
			let this = Pin::new(&mut **self);
			<$to>::poll_read_slice(this, cx, buf)
		}
	};
}

impl<T: AsyncRead + ?Sized> AsyncRead for &mut T {
	forward_impl_async_read!(T);
}

#[cfg(feature = "alloc")]
impl<T: AsyncRead + ?Sized> AsyncRead for alloc::boxed::Box<T> {
	forward_impl_async_read!(T);
}
