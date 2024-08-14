use crate::buf::BufMut;
use crate::error::{IoError, ReadError};

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

impl<T: Read + ?Sized> ReadExt for T {}

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

impl<T: Read + ?Sized> Read for &mut T {
	forward_impl_read!(T);
}

#[cfg(feature = "alloc")]
impl<T: Read + ?Sized> Read for alloc::boxed::Box<T> {
	forward_impl_read!(T);
}
