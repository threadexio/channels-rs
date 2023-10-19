use core::future::Future;
use core::marker::Unpin;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::decouple;
use crate::buf::IoSliceRef;

/// This is the asynchronous version of [`Write`].
///
/// [`Write`]: crate::Write
pub trait AsyncWrite {
	/// The error returned by [`AsyncWrite::write_all()`] and [`AsyncWrite::flush()`].
	type Error;

	/// Poll the underlying writer and try to write some data to it.
	fn poll_write_all(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut IoSliceRef,
	) -> Poll<Result<(), Self::Error>>;

	/// Poll the underlying writer and try to flush it.
	fn poll_flush(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>>;

	/// Attempts to write an entire buffer `buf` to the writer.
	///
	/// This function returns a future that must be awaited for any work to
	/// happen.
	fn write_all<'a>(
		&'a mut self,
		buf: &'a mut IoSliceRef<'a>,
	) -> WriteAll<'a, Self>
	where
		Self: Unpin,
	{
		WriteAll::new(self, buf)
	}

	/// Flush the writer ensuring all bytes reach their destination.
	///
	/// This function returns a future that must be awaited for any work to
	/// happen.
	fn flush(&mut self) -> Flush<'_, Self>
	where
		Self: Unpin,
	{
		Flush::new(self)
	}
}

#[doc(hidden)]
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct WriteAll<'a, W>
where
	W: AsyncWrite + Unpin + ?Sized,
{
	writer: &'a mut W,
	buf: &'a mut IoSliceRef<'a>,
}

impl<'a, W> WriteAll<'a, W>
where
	W: AsyncWrite + Unpin + ?Sized,
{
	pub(self) fn new(
		writer: &'a mut W,
		buf: &'a mut IoSliceRef<'a>,
	) -> Self {
		Self { writer, buf }
	}
}

impl<'a, W> Future for WriteAll<'a, W>
where
	W: AsyncWrite + Unpin + ?Sized,
{
	type Output = Result<(), W::Error>;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		let buf = decouple!(*self.buf; as mut);
		Pin::new(&mut *self.writer).poll_write_all(cx, buf)
	}
}

#[doc(hidden)]
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct Flush<'a, W>
where
	W: AsyncWrite + Unpin + ?Sized,
{
	writer: &'a mut W,
}

impl<'a, W> Flush<'a, W>
where
	W: AsyncWrite + Unpin + ?Sized,
{
	pub(self) fn new(writer: &'a mut W) -> Self {
		Self { writer }
	}
}

impl<'a, W> Future for Flush<'a, W>
where
	W: AsyncWrite + Unpin + ?Sized,
{
	type Output = Result<(), W::Error>;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		Pin::new(&mut *self.writer).poll_flush(cx)
	}
}

/// Types that can be converted to [`AsyncWrite`]ers.
pub trait IntoAsyncWriter<T: AsyncWrite> {
	/// Convert this type into an asynchronous writer `T`.
	fn into_async_writer(self) -> T;
}

impl<T: AsyncWrite> IntoAsyncWriter<T> for T {
	fn into_async_writer(self) -> T {
		self
	}
}
