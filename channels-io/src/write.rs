use crate::{
	IoError, WriteBuf, WriteError, WriteTransaction,
	WriteTransactionKind,
};

/// This trait allows writing bytes to a writer.
///
/// Types implementing this trait are called "writers".
///
/// Writers have only one method, [`Write::write()`] which will attempt to write
/// some bytes to the sink.
pub trait Write {
	/// Error type for [`Write::write()`].
	type Error: WriteError;

	/// Write `buf` to the writer.
	///
	/// Upon return, this function must guarantee that either: a) `buf` has been
	/// fully written, `buf.has_remaining()` should return `false`. b)
	/// an error has occurred that _cannot_ be handled immediately.
	///
	/// If `buf` has been written to the writer, then this function must return
	/// with [`Ok(())`](Ok). In any other case it must return an [`Err`].
	fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
		default_write(self, buf)
	}

	/// Flush this writer ensuring all bytes reach their destination.
	///
	/// Upon return, this function must ensure that all bytes written to the
	/// writer by previous calls to [`Write::write()`] have reached their
	/// destination.
	fn flush(&mut self) -> Result<(), Self::Error> {
		default_flush(self)
	}

	/// Write some bytes from `buf` to the writer.
	///
	/// This function is the lower level building block of [`write()`]. It writes
	/// bytes from `buf` and reports back to the caller how many bytes it wrote.
	/// [`write()`] should, usually, be preferred.
	///
	/// [`write()`]: fn@Write::write
	fn write_slice(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error>;

	/// Flush this writer once ensuring all bytes reach their destination.
	///
	/// This function is the lower level building block of [`flush()`]. It flushes
	/// the writer only once. [`flush()`] should, usually, be preferred.
	///
	/// [`flush()`]: fn@Write::flush
	fn flush_once(&mut self) -> Result<(), Self::Error>;

	/// Create a "by reference" adapter that takes the current instance of [`Write`]
	/// by mutable reference.
	fn by_ref(&mut self) -> &mut Self
	where
		Self: Sized,
	{
		self
	}

	/// Create a transaction that uses this instance of [`Write`].
	///
	/// This is a convenience wrapper for: [`WriteTransaction::new()`]
	fn transaction(
		self,
		kind: WriteTransactionKind,
	) -> WriteTransaction<'_, Self>
	where
		Self: Sized,
	{
		WriteTransaction::new(self, kind)
	}
}

fn default_write<T>(
	writer: &mut T,
	buf: &[u8],
) -> Result<(), T::Error>
where
	T: Write + ?Sized,
{
	let mut buf = WriteBuf::new(buf);

	while !buf.remaining().is_empty() {
		match writer.write_slice(buf.remaining()) {
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
	T: Write + ?Sized,
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

		fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
			<$to>::write(self, buf)
		}

		fn flush(&mut self) -> Result<(), Self::Error> {
			<$to>::flush(self)
		}

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
