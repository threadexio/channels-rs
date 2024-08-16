use core::pin::Pin;
use core::task::{ready, Context, Poll};

use crate::buf::Buf;
use crate::error::{IoError, WriteError};

#[cfg(feature = "alloc")]
use crate::transaction::{
	WriteTransactionKind, WriteTransactionVariant,
};

mod flush;
mod write_buf;
mod write_buf_all;

use self::flush::Flush;
use self::write_buf::WriteBuf;
use self::write_buf_all::WriteBufAll;

/// This trait allows writing bytes to a writer.
///
/// Types implementing this trait are called "writers".
pub trait Write {
	/// Error type for IO operations involving the writer.
	type Error: WriteError;

	/// Write some bytes from `buf` to the writer.
	///
	/// This function is the lower level building block of [`write_buf()`]. It writes
	/// bytes from `buf` and reports back to the caller how many bytes it wrote.
	/// [`write_buf()`] should, usually, be preferred.
	///
	/// [`write_buf()`]: WriteExt::write_buf
	fn write_slice(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error>;

	/// Flush this writer once ensuring all bytes reach their destination.
	///
	/// This function is the lower level building block of [`flush()`]. It flushes
	/// the writer only once. [`flush()`] should, usually, be preferred.
	///
	/// [`flush()`]: WriteExt::flush
	fn flush_once(&mut self) -> Result<(), Self::Error>;
}

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

/// Write bytes to a writer.
///
/// Extension trait for all [`Write`] types.
pub trait WriteExt: Write {
	/// Write `buf` to the writer advancing it appropriately.
	fn write_buf<B>(&mut self, buf: B) -> Result<(), Self::Error>
	where
		B: Buf,
	{
		write_buf(self, buf)
	}

	/// Write `buf` to the writer advancing it until all of it has been written.
	///
	/// This method will try to write `buf` repeatedly until either a) `buf` has
	/// no more data, b) an error occurs, c) the writer cannot accept any more bytes.
	fn write_buf_all<B>(&mut self, buf: B) -> Result<(), Self::Error>
	where
		B: Buf,
	{
		write_buf_all(self, buf)
	}

	/// Flush this writer ensuring all bytes reach their destination.
	fn flush(&mut self) -> Result<(), Self::Error> {
		flush(self)
	}

	/// Create a "by reference" adapter that takes the current instance of [`Write`]
	/// by mutable reference.
	fn by_ref(&mut self) -> &mut Self
	where
		Self: Sized,
	{
		self
	}

	/// Create a transaction that uses this writer.
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
		WriteBuf::new(self, buf)
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
		WriteBufAll::new(self, buf)
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

impl<T: Write + ?Sized> WriteExt for T {}

impl<T: AsyncWrite + ?Sized> AsyncWriteExt for T {}

fn write_buf<T, B>(writer: &mut T, mut buf: B) -> Result<(), T::Error>
where
	T: WriteExt + ?Sized,
	B: Buf,
{
	if !buf.has_remaining() {
		return Ok(());
	}

	loop {
		match writer.write_slice(buf.chunk()) {
			Ok(0) => return Err(T::Error::write_zero()),
			Ok(n) => {
				buf.advance(n);
				return Ok(());
			},
			Err(e) if e.should_retry() => continue,
			Err(e) => return Err(e),
		}
	}
}

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

fn write_buf_all<T, B>(
	writer: &mut T,
	mut buf: B,
) -> Result<(), T::Error>
where
	T: WriteExt + ?Sized,
	B: Buf,
{
	while buf.has_remaining() {
		match writer.write_slice(buf.chunk()) {
			Ok(0) => return Err(T::Error::write_zero()),
			Ok(n) => buf.advance(n),
			Err(e) if e.should_retry() => continue,
			Err(e) => return Err(e),
		}
	}

	Ok(())
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

fn flush<T>(writer: &mut T) -> Result<(), T::Error>
where
	T: WriteExt + ?Sized,
{
	loop {
		match writer.flush_once() {
			Ok(()) => break Ok(()),
			Err(e) if e.should_retry() => continue,
			Err(e) => break Err(e),
		}
	}
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

macro_rules! forward_impl_write {
	($to:ty) => {
		type Error = <$to>::Error;

		fn write_slice(
			&mut self,
			buf: &[u8],
		) -> Result<usize, Self::Error> {
			<$to>::write_slice(self, buf)
		}

		fn flush_once(&mut self) -> Result<(), Self::Error> {
			<$to>::flush_once(self)
		}
	};
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

impl<T: Write + ?Sized> Write for &mut T {
	forward_impl_write!(T);
}

impl<T: AsyncWrite + Unpin + ?Sized> AsyncWrite for &mut T {
	forward_impl_async_write!(T);
}

#[cfg(feature = "alloc")]
impl<T: Write + ?Sized> Write for alloc::boxed::Box<T> {
	forward_impl_write!(T);
}

#[cfg(feature = "alloc")]
impl<T: AsyncWrite + Unpin + ?Sized> AsyncWrite
	for alloc::boxed::Box<T>
{
	forward_impl_async_write!(T);
}
