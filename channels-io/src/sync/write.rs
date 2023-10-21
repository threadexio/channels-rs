use core::task::Poll;

use crate::buf::IoSliceRef;

/// The Write trait allows writing bytes to some place.
///
/// Types which implement this trait are called 'writers'.
pub trait Write {
	/// The error returned by [`Write::write_all()`] and [`Write::flush()`].
	type Error;

	/// Attempt to write an entire buffer `buf` into the writer.
	///
	/// This function returns a result wrapped in [`Poll`] because it is not
	/// specified whether writers will block or not. In the event a reader is
	/// non-blocking this function will return [`Poll::Pending`]. In that case
	/// callers should take care to call the function again at a later time to
	/// continue. Ideally, non-blocking writers should be using [`AsyncWrite`]
	/// instead, but on environment where a fully fledged executor is not
	/// available or needed this can be used instead.
	///
	/// If you know for certain that the underlying writer will always block,
	/// you can safely destruct [`Poll`] away with [`unwrap`].
	///
	/// [`AsyncWrite`]: crate::AsyncWrite
	/// [`unwrap`]: crate::PollExt::unwrap()
	fn write_all(
		&mut self,
		buf: &mut IoSliceRef,
	) -> Poll<Result<(), Self::Error>>;

	/// Flush the writer ensuring all bytes reach their destination.
	fn flush(&mut self) -> Poll<Result<(), Self::Error>>;
}

/// Types that can be converted to [`Write`]ers.
pub trait IntoWriter<T: Write> {
	/// Convert this type into a writer `T`.
	fn into_writer(self) -> T;
}

impl<T: Write> IntoWriter<T> for T {
	fn into_writer(self) -> T {
		self
	}
}
