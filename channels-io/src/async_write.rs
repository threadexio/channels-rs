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
	/// Error type for IO operations involving the writer.
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
	fn poll_write_buf<B>(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut B,
	) -> Poll<Result<(), Self::Error>>
	where
		B: Buf + ?Sized,
	{
		poll_write_buf(self, cx, buf)
	}

	/// Poll the writer and try to write `buf` to it.
	fn poll_write_buf_all<B>(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut B,
	) -> Poll<Result<(), Self::Error>>
	where
		B: Buf + ?Sized,
	{
		poll_write_buf_all(self, cx, buf)
	}

	/// Poll the writer and try to flush it.
	fn poll_flush(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		poll_flush(self, cx)
	}

	/// Asynchronously write `buf` to the writer advancing it appropriately
	///
	/// This function behaves in the same way as [`write_buf()`] except that
	/// it returns a [`Future`] that must be `.await`ed.
	///
	/// [`write_buf()`]: crate::WriteExt::write_buf
	/// [`Future`]: core::future::Future
	fn write_buf<B>(&mut self, buf: B) -> WriteBuf<'_, Self, B>
	where
		B: Buf + Unpin,
		Self: Unpin,
	{
		write_buf(self, buf)
	}

	/// Asynchronously write `buf` to the writer advancing it until all of it has been written.
	///
	/// This function behaves in the same way as [`write_buf_all()`] except that
	/// it returns a [`Future`] that must be `.await`ed.
	///
	/// [`write_buf_all()`]: crate::WriteExt::write_buf_all
	/// [`Future`]: core::future::Future
	fn write_buf_all<B>(&mut self, buf: B) -> WriteBufAll<'_, Self, B>
	where
		B: Buf + Unpin,
		Self: Unpin,
	{
		write_buf_all(self, buf)
	}

	/// Asynchronously flush the writer.
	///
	/// This function behaves in the same way as [`flush()`] except that
	/// it returns a [`Future`] that must be `.await`ed.
	///
	/// [`flush()`]: crate::WriteExt::flush
	/// [`Future`]: core::future::Future
	fn flush(&mut self) -> Flush<'_, Self>
	where
		Self: Unpin,
	{
		flush(self)
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

fn poll_write_buf<T, B>(
	mut writer: Pin<&mut T>,
	cx: &mut Context,
	buf: &mut B,
) -> Poll<Result<(), T::Error>>
where
	T: AsyncWriteExt + ?Sized,
	B: Buf + ?Sized,
{
	use Poll::Ready;

	if !buf.has_remaining() {
		return Ready(Ok(()));
	}

	loop {
		match ready!(writer
			.as_mut()
			.poll_write_slice(cx, buf.chunk()))
		{
			Ok(0) => return Ready(Err(T::Error::write_zero())),
			Ok(n) => {
				buf.advance(n);
				return Ready(Ok(()));
			},
			Err(e) if e.should_retry() => continue,
			Err(e) => return Ready(Err(e)),
		}
	}
}

fn poll_write_buf_all<T, B>(
	mut writer: Pin<&mut T>,
	cx: &mut Context,
	buf: &mut B,
) -> Poll<Result<(), T::Error>>
where
	T: AsyncWriteExt + ?Sized,
	B: Buf + ?Sized,
{
	use Poll::Ready;

	while buf.has_remaining() {
		match ready!(writer
			.as_mut()
			.poll_write_slice(cx, buf.chunk()))
		{
			Ok(0) => return Ready(Err(T::Error::write_zero())),
			Ok(n) => buf.advance(n),
			Err(e) if e.should_retry() => continue,
			Err(e) => return Ready(Err(e)),
		}
	}

	Ready(Ok(()))
}

fn poll_flush<T>(
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

fn write_buf<T, B>(writer: &mut T, buf: B) -> WriteBuf<'_, T, B>
where
	T: AsyncWriteExt + Unpin + ?Sized,
	B: Buf + Unpin,
{
	WriteBuf { writer, buf }
}

fn write_buf_all<T, B>(
	writer: &mut T,
	buf: B,
) -> WriteBufAll<'_, T, B>
where
	T: AsyncWriteExt + Unpin + ?Sized,
	B: Buf + Unpin,
{
	WriteBufAll { writer, buf }
}

fn flush<T>(writer: &mut T) -> Flush<'_, T>
where
	T: AsyncWriteExt + Unpin + ?Sized,
{
	Flush { writer }
}

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct WriteBuf<'a, T, B>
where
	T: AsyncWriteExt + Unpin + ?Sized,
	B: Buf + Unpin,
{
	writer: &'a mut T,
	buf: B,
}

impl<'a, T, B> Future for WriteBuf<'a, T, B>
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
		Pin::new(&mut **writer).poll_write_buf(cx, buf)
	}
}

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct WriteBufAll<'a, T, B>
where
	T: AsyncWriteExt + Unpin + ?Sized,
	B: Buf + Unpin,
{
	writer: &'a mut T,
	buf: B,
}

impl<'a, T, B> Future for WriteBufAll<'a, T, B>
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
		Pin::new(&mut **writer).poll_write_buf_all(cx, buf)
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
			<$to>::poll_write_slice(this, cx, buf)
		}

		fn poll_flush_once(
			mut self: Pin<&mut Self>,
			cx: &mut Context,
		) -> Poll<Result<(), Self::Error>> {
			let this = Pin::new(&mut **self);
			<$to>::poll_flush_once(this, cx)
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
