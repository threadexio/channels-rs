use core::pin::Pin;
use core::task::{ready, Context, Poll};

use crate::buf::BufMut;
use crate::error::{IoError, ReadError};

mod read_buf;
mod read_buf_all;

use self::read_buf::ReadBuf;
use self::read_buf_all::ReadBufAll;

/// This trait allows reading bytes from a source.
///
/// Types implementing this trait are called "readers".
pub trait Read {
	/// Error type for IO operations involving the reader.
	type Error: ReadError;

	/// Read some bytes into the slice `buf`.
	///
	/// This function is the lower level building block of the other `read_*` methods.
	/// It reads bytes into `buf` and reports back to the caller how many bytes it read.
	fn read_slice(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, Self::Error>;
}

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

/// Read bytes from a reader.
///
/// Extension trait for all [`Read`] types.
pub trait ReadExt: Read {
	/// Read some bytes into `buf` advancing it appropriately.
	fn read_buf<B>(&mut self, buf: B) -> Result<(), Self::Error>
	where
		B: BufMut,
	{
		read_buf(self, buf)
	}

	/// Read bytes into `buf` advancing it until it is full.
	///
	/// This method will try to read bytes into `buf` repeatedly until either a)
	/// `buf` has been filled, b) an error occurs or c) the reader reaches EOF.
	fn read_buf_all<B>(&mut self, buf: B) -> Result<(), Self::Error>
	where
		B: BufMut,
	{
		read_buf_all(self, buf)
	}

	/// Create a "by reference" adapter that takes the current instance of [`Read`]
	/// by mutable reference.
	fn by_ref(&mut self) -> &mut Self
	where
		Self: Sized,
	{
		self
	}
}

/// This trait is the asynchronous version of [`ReadExt`].
///
/// Extension trait for all [`AsyncRead`] types.
///
/// [`ReadExt`]: crate::ReadExt
pub trait AsyncReadExt: AsyncRead {
	/// Poll the reader and try to read some bytes into `buf`.
	fn poll_read_buf<B>(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut B,
	) -> Poll<Result<(), Self::Error>>
	where
		B: BufMut + ?Sized,
	{
		poll_read_buf(self, cx, buf)
	}

	/// Poll the reader and try to read some bytes into `buf`.
	fn poll_read_buf_all<B>(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut B,
	) -> Poll<Result<(), Self::Error>>
	where
		B: BufMut + ?Sized,
	{
		poll_read_buf_all(self, cx, buf)
	}

	/// Asynchronously read some bytes into `buf` advancing it appropriately.
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

	/// Asynchronously read bytes into `buf` advancing it until it is full.
	///
	/// This function behaves in the same way as [`read_buf_all()`] except that it
	/// returns a [`Future`] that must be `.await`ed.
	///
	/// [`read_buf_all()`]: crate::ReadExt::read_buf_all
	/// [`Future`]: core::future::Future
	fn read_buf_all<B>(&mut self, buf: B) -> ReadBufAll<'_, Self, B>
	where
		B: BufMut + Unpin,
		Self: Unpin,
	{
		ReadBufAll::new(self, buf)
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

impl<T: Read + ?Sized> ReadExt for T {}

impl<T: AsyncRead + ?Sized> AsyncReadExt for T {}

fn read_buf<T, B>(reader: &mut T, mut buf: B) -> Result<(), T::Error>
where
	T: ReadExt + ?Sized,
	B: BufMut,
{
	if !buf.has_remaining_mut() {
		return Ok(());
	}

	loop {
		match reader.read_slice(buf.chunk_mut()) {
			Ok(0) => return Err(T::Error::eof()),
			Ok(n) => {
				buf.advance_mut(n);
				return Ok(());
			},
			Err(e) if e.should_retry() => continue,
			Err(e) => return Err(e),
		}
	}
}

fn poll_read_buf<T, B>(
	mut reader: Pin<&mut T>,
	cx: &mut Context,
	buf: &mut B,
) -> Poll<Result<(), T::Error>>
where
	T: AsyncReadExt + ?Sized,
	B: BufMut + ?Sized,
{
	use Poll::Ready;

	if !buf.has_remaining_mut() {
		return Ready(Ok(()));
	}

	loop {
		match ready!(reader
			.as_mut()
			.poll_read_slice(cx, buf.chunk_mut()))
		{
			Ok(0) => return Ready(Err(T::Error::eof())),
			Ok(n) => {
				buf.advance_mut(n);
				return Ready(Ok(()));
			},
			Err(e) if e.should_retry() => continue,
			Err(e) => return Ready(Err(e)),
		}
	}
}

fn read_buf_all<T, B>(
	reader: &mut T,
	mut buf: B,
) -> Result<(), T::Error>
where
	T: ReadExt + ?Sized,
	B: BufMut,
{
	while buf.has_remaining_mut() {
		match reader.read_slice(buf.chunk_mut()) {
			Ok(0) => return Err(T::Error::eof()),
			Ok(n) => buf.advance_mut(n),
			Err(e) if e.should_retry() => continue,
			Err(e) => return Err(e),
		}
	}

	Ok(())
}

fn poll_read_buf_all<T, B>(
	mut reader: Pin<&mut T>,
	cx: &mut Context,
	buf: &mut B,
) -> Poll<Result<(), T::Error>>
where
	T: AsyncReadExt + ?Sized,
	B: BufMut + ?Sized,
{
	use Poll::Ready;

	while buf.has_remaining_mut() {
		match ready!(reader
			.as_mut()
			.poll_read_slice(cx, buf.chunk_mut()))
		{
			Ok(0) => return Ready(Err(T::Error::eof())),
			Ok(n) => buf.advance_mut(n),
			Err(e) if e.should_retry() => continue,
			Err(e) => return Ready(Err(e)),
		}
	}

	Ready(Ok(()))
}

macro_rules! forward_impl_read {
	($to:ty) => {
		type Error = <$to>::Error;

		fn read_slice(
			&mut self,
			buf: &mut [u8],
		) -> Result<usize, Self::Error> {
			<$to>::read_slice(self, buf)
		}
	};
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

impl<T: Read + ?Sized> Read for &mut T {
	forward_impl_read!(T);
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

#[cfg(feature = "alloc")]
impl<T: Read + ?Sized> Read for alloc::boxed::Box<T> {
	forward_impl_read!(T);
}
