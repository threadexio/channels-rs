use crate::buf::Buf;
use crate::error::{IoError, WriteError};

#[cfg(feature = "alloc")]
use crate::transaction::{
	WriteTransactionKind, WriteTransactionVariant,
};

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

/// Write bytes to a writer.
///
/// Extension trait for all [`Write`] types.
pub trait WriteExt: Write {
	/// Write `buf` to the writer.
	///
	/// This method will try to write bytes from `buf` to the writer repeatedly
	/// until either a) `buf` has no more bytes left, b) an error occurs or c)
	/// the writer reaches EOF. If the writer reaches EOF, this method will return
	/// an error.
	fn write_buf<B>(&mut self, buf: B) -> Result<(), Self::Error>
	where
		B: Buf,
	{
		default_write_buf(self, buf)
	}

	/// Flush this writer ensuring all bytes reach their destination.
	///
	/// Upon return, this function must ensure that all bytes written to the
	/// writer  have reached their destination.
	fn flush(&mut self) -> Result<(), Self::Error> {
		default_flush(self)
	}

	/// Create a "by reference" adapter that takes this writer by mutable reference.
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

impl<T: Write + ?Sized> WriteExt for T {}

fn default_write_buf<T, B>(
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

fn default_flush<T>(writer: &mut T) -> Result<(), T::Error>
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

impl<T: Write + ?Sized> Write for &mut T {
	forward_impl_write!(T);
}

#[cfg(feature = "alloc")]
impl<T: Write + ?Sized> Write for alloc::boxed::Box<T> {
	forward_impl_write!(T);
}
