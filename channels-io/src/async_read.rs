use core::future::Future;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use crate::buf::BufMut;
use crate::error::{IoError, ReadError};

/// This trait is the asynchronous version of [`Read`].
///
/// [`Read`]: crate::Read
pub trait AsyncRead {
	/// Error type for IO operations involving the reader.
	type Error: ReadError;

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

/// This trait is the asynchronous version of [`ReadExt`].
///
/// Extension trait for all [`AsyncRead`] types.
///
/// [`ReadExt`]: crate::ReadExt
pub trait AsyncReadExt: AsyncRead {
	/// Poll the reader and try to read some bytes into `buf`.
	///
	/// This method reads bytes into the unfilled part of `buf` and advances the
	/// it accordingly.
	fn poll_read_buf<B>(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut B,
	) -> Poll<Result<(), Self::Error>>
	where
		B: BufMut + ?Sized,
	{
		default_poll_read_buf(self, cx, buf)
	}

	/// Asynchronously read some bytes into `buf`.
	///
	/// This function behaves in the same way as [`read_buf()`] except that it
	/// returns a [`Future`] that must be `.await`ed.
	///
	/// [`read_buf()`]: crate::ReadExt::read_buf
	/// [`Future`]: core::future::Future
	fn read_buf<B>(&mut self, buf: B) -> ReadBuf<'_, Self, B>
	where
		B: BufMut + Unpin,
		Self: Unpin,
	{
		ReadBuf::new(self, buf)
	}

	/// Create a "by reference" adapter that takes the current instance of [`AsyncRead`]
	/// by mutable reference.
	fn by_ref(&mut self) -> &mut Self
	where
		Self: Sized,
	{
		self
	}
}

impl<T: AsyncRead + ?Sized> AsyncReadExt for T {}

fn default_poll_read_buf<T, B>(
	mut reader: Pin<&mut T>,
	cx: &mut Context,
	buf: &mut B,
) -> Poll<Result<(), T::Error>>
where
	T: AsyncReadExt + ?Sized,
	B: BufMut + ?Sized,
{
	while buf.has_remaining_mut() {
		match ready!(reader
			.as_mut()
			.poll_read_slice(cx, buf.chunk_mut()))
		{
			Ok(0) => return Poll::Ready(Err(T::Error::eof())),
			Ok(n) => buf.advance_mut(n),
			Err(e) if e.should_retry() => continue,
			Err(e) => return Poll::Ready(Err(e)),
		}
	}

	Poll::Ready(Ok(()))
}

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct ReadBuf<'a, T, B>
where
	T: AsyncReadExt + Unpin + ?Sized,
	B: BufMut + Unpin + ?Sized,
{
	reader: &'a mut T,
	buf: B,
}

impl<'a, T, B> ReadBuf<'a, T, B>
where
	T: AsyncReadExt + Unpin + ?Sized,
	B: BufMut + Unpin,
{
	fn new(reader: &'a mut T, buf: B) -> Self {
		Self { reader, buf }
	}
}

impl<'a, T, B> Future for ReadBuf<'a, T, B>
where
	T: AsyncReadExt + Unpin + ?Sized,
	B: BufMut + Unpin + ?Sized,
{
	type Output = Result<(), T::Error>;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Self::Output> {
		let Self { ref mut reader, ref mut buf } = *self;
		Pin::new(&mut **reader).poll_read_buf(cx, buf)
	}
}

macro_rules! forward_impl_async_read {
	($to:ty) => {
		type Error = <$to>::Error;

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

impl<T: AsyncRead + Unpin + ?Sized> AsyncRead for &mut T {
	forward_impl_async_read!(T);
}

#[cfg(feature = "alloc")]
impl<T: AsyncRead + Unpin + ?Sized> AsyncRead
	for alloc::boxed::Box<T>
{
	forward_impl_async_read!(T);
}
