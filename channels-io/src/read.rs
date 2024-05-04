/// This trait allows reading bytes from a source.
///
/// Types implementing this trait are called "readers".
///
/// Readers have only one method, [`Read::read()`] which will attempt to read some
/// bytes into the provided buffer.
pub trait Read {
	/// Error type for [`Read::read()`].
	type Error;

	/// Read some bytes into `buf`.
	///
	/// Upon return, this function must guarantee that either: a) `buf` has no
	/// more space to fill, `buf.has_remaining_mut()` should return `false`. b)
	/// an error has occurred that _cannot_ be handled immediately.
	///
	/// If `buf` has been filled with data, then this function must return with
	/// [`Ok((())`](Ok). In any other case it must return an [`Err`].
	fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error>;
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
	};
}

impl<T: Read + ?Sized> Read for &mut T {
	forward_impl_read!(T);
}

#[cfg(feature = "alloc")]
impl<T: Read + ?Sized> Read for alloc::boxed::Box<T> {
	forward_impl_read!(T);
}
