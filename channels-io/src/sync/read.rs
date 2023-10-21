use core::task::Poll;

use crate::buf::IoSliceMut;

/// The Read trait allows reading bytes from a source.
///
/// Types which implement this trait are called 'readers'.
pub trait Read {
	/// The error type returned by [`Read::read_all()`].
	type Error;

	/// Read as many bytes as is required to fill `buf`.
	///
	/// This function returns a result wrapped in [`Poll`] because it is not
	/// specified whether readers will block or not. In the event a reader is
	/// non-blocking this function will return [`Poll::Pending`]. In that case
	/// callers should take care to call the function again at a later time to
	/// continue. Ideally, non-blocking readers should be using [`AsyncRead`]
	/// instead, but on environment where a fully fledged executor is not
	/// available or needed this can be used instead.
	///
	/// If you know for certain that the underlying reader will always block,
	/// you can safely destruct [`Poll`] away with [`unwrap`].
	///
	/// [`AsyncRead`]: crate::AsyncRead
	/// [`unwrap`]: crate::PollExt::unwrap()
	fn read_all(
		&mut self,
		buf: &mut IoSliceMut,
	) -> Poll<Result<(), Self::Error>>;
}

impl<T: Read + ?Sized> Read for &mut T {
	type Error = T::Error;

	fn read_all(
		&mut self,
		buf: &mut IoSliceMut,
	) -> Poll<Result<(), Self::Error>> {
		(**self).read_all(buf)
	}
}

/// Types that can be converted to [`Read`]ers.
pub trait IntoReader<T: Read> {
	/// Convert this type into a reader `T`.
	fn into_reader(self) -> T;
}

impl<T: Read> IntoReader<T> for T {
	fn into_reader(self) -> T {
		self
	}
}
