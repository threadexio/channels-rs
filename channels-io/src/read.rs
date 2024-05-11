use crate::{ReadBuf, ReadError};

/// This trait allows reading bytes from a source.
///
/// Types implementing this trait are called "readers".
///
/// Readers have only one method, [`Read::read()`] which will attempt to read some
/// bytes into the provided buffer.
pub trait Read {
	/// Error type for [`Read::read()`].
	type Error: ReadError;

	/// Read some bytes into `buf`.
	///
	/// Upon return, this function must guarantee that either: a) `buf` has no
	/// more space to fill, `buf.has_remaining_mut()` should return `false`. b)
	/// an error has occurred that _cannot_ be handled immediately.
	///
	/// If `buf` has been filled with data, then this function must return with
	/// [`Ok((())`](Ok). In any other case it must return an [`Err`].
	fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
		default_read(self, buf)
	}

	/// Read some bytes into the slice `buf`.
	///
	/// This function is the lower level building block of [`read()`]. It reads
	/// bytes into `buf` and reports back to the caller how many bytes it read.
	/// [`read()`] should, usually, be preferred.
	///
	/// [`read()`]: fn@Read::read
	fn read_slice(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, Self::Error>;
}

fn default_read<T>(
	reader: &mut T,
	buf: &mut [u8],
) -> Result<(), T::Error>
where
	T: Read + ?Sized,
{
	let mut buf = ReadBuf::new(buf);

	while !buf.unfilled().is_empty() {
		match reader.read_slice(buf.unfilled_mut()) {
			Ok(0) => return Err(T::Error::eof()),
			Ok(n) => buf.advance(n),
			Err(e) if e.should_retry() => continue,
			Err(e) => return Err(e),
		}
	}

	Ok(())
}

macro_rules! forward_impl_read {
	($to:ty) => {
		type Error = <$to>::Error;

		fn read(
			&mut self,
			buf: &mut [u8],
		) -> Result<(), Self::Error> {
			<$to>::read(self, buf)
		}

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
