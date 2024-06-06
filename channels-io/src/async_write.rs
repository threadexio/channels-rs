use core::future::Future;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use crate::error::{IoError, WriteError};
use crate::WriteBuf;

#[cfg(feature = "alloc")]
use crate::transaction::{
	AsyncWriteTransaction, WriteTransactionKind,
};

/// This trait is the asynchronous version of [`Write`].
///
/// [`Write`]: crate::Write
pub trait AsyncWrite: Unpin {
	/// Error type for [`write()`] and [`flush()`].
	///
	/// [`write()`]: AsyncWrite::write
	/// [`flush()`]: AsyncWrite::flush
	type Error: WriteError;

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
	) -> Poll<Result<(), Self::Error>> {
		default_poll_write(self, cx, buf)
	}

	/// Poll the writer and try to write some bytes from `buf` to it.
	///
	/// This method writes bytes from `buf` and reports back how many bytes it
	/// wrote.
	fn poll_write_slice(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, Self::Error>>;

	/// Poll the writer and try to flush it.
	fn poll_flush(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		default_poll_flush(self, cx)
	}

	/// Poll the writer and try to flush it only once.
	fn poll_flush_once(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>>;

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
	/// This is a convenience wrapper for: [`AsyncWriteTransaction::new()`]
	#[cfg(feature = "alloc")]
	fn transaction(
		self,
		kind: WriteTransactionKind,
	) -> AsyncWriteTransaction<'_, Self>
	where
		Self: Sized,
	{
		AsyncWriteTransaction::new(self, kind)
	}
}

fn default_poll_write<T>(
	mut writer: Pin<&mut T>,
	cx: &mut Context,
	buf: &mut WriteBuf,
) -> Poll<Result<(), T::Error>>
where
	T: AsyncWrite + ?Sized,
{
	while !buf.remaining().is_empty() {
		match ready!(writer
			.as_mut()
			.poll_write_slice(cx, buf.remaining()))
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
		Pin::new(&mut **writer).poll_write(cx, buf)
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
		let Self { ref mut writer, .. } = *self;
		Pin::new(&mut **writer).poll_flush(cx)
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
			let this = Pin::new(&mut **self);
			T::poll_write(this, cx, buf)
		}

		fn poll_write_slice(
			mut self: Pin<&mut Self>,
			cx: &mut Context,
			buf: &[u8],
		) -> Poll<Result<usize, Self::Error>> {
			let this = Pin::new(&mut **self);
			T::poll_write_slice(this, cx, buf)
		}

		fn poll_flush(
			mut self: Pin<&mut Self>,
			cx: &mut Context,
		) -> Poll<Result<(), Self::Error>> {
			let this = Pin::new(&mut **self);
			T::poll_flush(this, cx)
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

impl<T: AsyncWrite + ?Sized> AsyncWrite for &mut T {
	forward_impl_async_write!(T);
}

#[cfg(feature = "alloc")]
impl<T: AsyncWrite + ?Sized> AsyncWrite for alloc::boxed::Box<T> {
	forward_impl_async_write!(T);
}
