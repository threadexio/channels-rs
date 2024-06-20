use core::future::Future;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use crate::buf::Buf;
use crate::error::{IoError, WriteError};

#[cfg(feature = "alloc")]
use crate::transaction::{
	WriteTransactionKind, WriteTransactionVariant,
};

/// This trait is the asynchronous version of [`Write`].
///
/// [`Write`]: crate::Write
pub trait AsyncWrite {
	/// Error type for [`write()`].
	///
	/// [`write()`]: fn@AsyncWriteExt::write
	type Error: WriteError;

	/// Poll the writer and try to write some bytes from `buf` to it.
	///
	/// This method writes bytes from `buf` and reports back how many bytes it
	/// wrote.
	fn poll_write_slice(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, Self::Error>>;

	/// Poll the writer and try to flush it only once.
	fn poll_flush_once(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>>;
}

/// This trait is the asynchronous version of [`WriteExt`].
///
/// Extension trait for all [`AsyncWrite`] types.
///
/// [`WriteExt`]: crate::WriteExt
pub trait AsyncWriteExt: AsyncWrite {
	/// Poll the writer and try to write `buf` to it.
	///
	/// This method writes bytes from `buf` to the writer and advances it
	/// accordingly.
	fn poll_write<B>(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: B,
	) -> Poll<Result<(), Self::Error>>
	where
		B: Buf,
	{
		default_poll_write(self, cx, buf)
	}

	/// Poll the writer and try to flush it.
	fn poll_flush(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		default_poll_flush(self, cx)
	}

	/// Asynchronously write `buf` to the writer.
	///
	/// This function behaves in the same way as [`write()`] except that
	/// it returns a [`Future`] that must be `.await`ed.
	///
	/// [`write()`]: fn@crate::WriteExt::write
	/// [`Future`]: core::future::Future
	fn write<B>(&mut self, buf: B) -> Write<'_, Self, B>
	where
		B: Buf + Unpin,
		Self: Unpin,
	{
		Write::new(self, buf)
	}

	/// Asynchronously flush the writer.
	///
	/// This function behaves in the same way as [`flush()`] except that
	/// it returns a [`Future`] that must be `.await`ed.
	///
	/// [`flush()`]: fn@crate::WriteExt::flush
	/// [`Future`]: core::future::Future
	fn flush(&mut self) -> Flush<'_, Self>
	where
		Self: Unpin,
	{
		Flush::new(self)
	}

	/// Create a "by reference" adapter that takes the current instance of [`AsyncWrite`]
	/// by mutable reference.
	fn by_ref(&mut self) -> &mut Self
	where
		Self: Sized,
	{
		self
	}

	/// Create a transaction that uses this instance of [`AsyncWrite`].
	///
	/// This is a convenience wrapper for: [`WriteTransactionVariant::new()`]
	#[cfg(feature = "alloc")]
	fn transaction(
		self,
		kind: WriteTransactionKind,
	) -> WriteTransactionVariant<'_, Self>
	where
		Self: Sized,
	{
		WriteTransactionVariant::new(self, kind)
	}
}

impl<T: AsyncWrite + ?Sized> AsyncWriteExt for T {}

fn default_poll_write<T, B>(
	mut writer: Pin<&mut T>,
	cx: &mut Context,
	mut buf: B,
) -> Poll<Result<(), T::Error>>
where
	T: AsyncWriteExt + ?Sized,
	B: Buf,
{
	while buf.has_remaining() {
		match ready!(writer
			.as_mut()
			.poll_write_slice(cx, buf.chunk()))
		{
			Ok(0) => return Poll::Ready(Err(T::Error::write_zero())),
			Ok(n) => buf.advance(n),
			Err(e) if e.should_retry() => continue,
			Err(e) => return Poll::Ready(Err(e)),
		}
	}

	Poll::Ready(Ok(()))
}

fn default_poll_flush<T>(
	mut writer: Pin<&mut T>,
	cx: &mut Context,
) -> Poll<Result<(), T::Error>>
where
	T: AsyncWrite + ?Sized,
{
	loop {
		match ready!(writer.as_mut().poll_flush_once(cx)) {
			Ok(()) => return Poll::Ready(Ok(())),
			Err(e) if e.should_retry() => continue,
			Err(e) => return Poll::Ready(Err(e)),
		}
	}
}

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct Write<'a, T, B>
where
	T: AsyncWriteExt + Unpin + ?Sized,
	B: Buf + Unpin,
{
	writer: &'a mut T,
	buf: B,
}

impl<'a, T, B> Write<'a, T, B>
where
	T: AsyncWriteExt + Unpin + ?Sized,
	B: Buf + Unpin,
{
	fn new(writer: &'a mut T, buf: B) -> Self {
		Self { writer, buf }
	}
}

impl<'a, T, B> Future for Write<'a, T, B>
where
	T: AsyncWriteExt + Unpin + ?Sized,
	B: Buf + Unpin,
{
	type Output = Result<(), T::Error>;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		let Self { ref mut writer, ref mut buf, .. } = *self;
		Pin::new(&mut **writer).poll_write(cx, &mut *buf)
	}
}

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct Flush<'a, T>
where
	T: AsyncWriteExt + Unpin + ?Sized,
{
	writer: &'a mut T,
}

impl<'a, T> Flush<'a, T>
where
	T: AsyncWriteExt + Unpin + ?Sized,
{
	fn new(writer: &'a mut T) -> Self {
		Self { writer }
	}
}

impl<'a, T> Future for Flush<'a, T>
where
	T: AsyncWriteExt + Unpin + ?Sized,
{
	type Output = Result<(), T::Error>;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		let Self { ref mut writer, .. } = *self;
		Pin::new(&mut **writer).poll_flush(cx)
	}
}

macro_rules! forward_impl_async_write {
	($to:ty) => {
		type Error = <$to>::Error;

		fn poll_write_slice(
			mut self: Pin<&mut Self>,
			cx: &mut Context,
			buf: &[u8],
		) -> Poll<Result<usize, Self::Error>> {
			let this = Pin::new(&mut **self);
			T::poll_write_slice(this, cx, buf)
		}

		fn poll_flush_once(
			mut self: Pin<&mut Self>,
			cx: &mut Context,
		) -> Poll<Result<(), Self::Error>> {
			let this = Pin::new(&mut **self);
			T::poll_flush_once(this, cx)
		}
	};
}

impl<T: AsyncWrite + Unpin + ?Sized> AsyncWrite for &mut T {
	forward_impl_async_write!(T);
}

#[cfg(feature = "alloc")]
impl<T: AsyncWrite + Unpin + ?Sized> AsyncWrite
	for alloc::boxed::Box<T>
{
	forward_impl_async_write!(T);
}
