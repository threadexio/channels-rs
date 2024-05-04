use core::future::Future;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use crate::WriteBuf;

/// This trait is the asynchronous version of [`Write`].
///
/// [`Write`]: crate::Write
pub trait AsyncWrite: Unpin {
	/// Error type for [`write()`] and [`flush()`].
	///
	/// [`write()`]: AsyncWrite::write
	/// [`flush()`]: AsyncWrite::flush
	type Error;

	/// Asynchronously write `buf` to the writer.
	///
	/// This function behaves in the same way as [`Write::write()`] except that
	/// it returns a [`Future`] that must be `.await`ed.
	///
	/// [`Write::write()`]: crate::Write::write
	/// [`Future`]: core::future::Future
	fn write<'a>(&'a mut self, buf: &'a [u8]) -> Write<'a, Self> {
		Write::new(self, buf)
	}

	/// Asynchronously flush the writer.
	///
	/// This function behaves in the same way as [`Write::flush()`] except that
	/// it returns a [`Future`] that must be `.await`ed.
	///
	/// [`Write::flush()`]: crate::Write::flush
	/// [`Future`]: core::future::Future
	fn flush(&mut self) -> Flush<'_, Self> {
		Flush::new(self)
	}

	/// Poll the writer and try to write `buf` to it.
	///
	/// This method writes bytes from `buf` to the writer and advances it
	/// accordingly.
	fn poll_write(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut WriteBuf,
	) -> Poll<Result<(), Self::Error>>;

	/// Poll the writer and try to flush it.
	fn poll_flush(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>>;
}

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct Write<'a, T: ?Sized> {
	writer: &'a mut T,
	buf: WriteBuf<'a>,
}

impl<'a, T: ?Sized> Write<'a, T> {
	fn new(writer: &'a mut T, buf: &'a [u8]) -> Self {
		Self { writer, buf: WriteBuf::new(buf) }
	}
}

impl<'a, T> Future for Write<'a, T>
where
	T: AsyncWrite + ?Sized,
{
	type Output = Result<(), T::Error>;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		let Self { ref mut writer, ref mut buf, .. } = *self;

		while !buf.remaining().is_empty() {
			ready!(Pin::new(&mut **writer).poll_write(cx, buf))?;
		}

		Poll::Ready(Ok(()))
	}
}

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct Flush<'a, T: ?Sized> {
	writer: &'a mut T,
}

impl<'a, T: ?Sized> Flush<'a, T> {
	fn new(writer: &'a mut T) -> Self {
		Self { writer }
	}
}

impl<'a, T> Future for Flush<'a, T>
where
	T: AsyncWrite + ?Sized,
{
	type Output = Result<(), T::Error>;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		Pin::new(&mut self.writer).poll_flush(cx)
	}
}

macro_rules! forward_impl_async_write {
	($to:ty) => {
		type Error = <$to>::Error;

		fn poll_write(
			mut self: Pin<&mut Self>,
			cx: &mut Context,
			buf: &mut WriteBuf,
		) -> Poll<Result<(), Self::Error>> {
			T::poll_write(Pin::new(&mut **self), cx, buf)
		}

		fn poll_flush(
			mut self: Pin<&mut Self>,
			cx: &mut Context,
		) -> Poll<Result<(), Self::Error>> {
			T::poll_flush(Pin::new(&mut **self), cx)
		}
	};
}

impl<T: AsyncWrite + ?Sized> AsyncWrite for &mut T {
	forward_impl_async_write!(T);
}

#[cfg(feature = "alloc")]
impl<T: AsyncWrite + ?Sized> AsyncWrite for alloc::boxed::Box<T> {
	forward_impl_async_write!(T);
}
