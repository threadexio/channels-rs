use core::future::Future;

use crate::Buf;

/// This is the asynchronous version of [`Write`].
///
/// [`Write`]: crate::Write
pub trait AsyncWrite {
	/// The error returned by [`AsyncWrite::write_all()`] and [`AsyncWrite::flush()`].
	type Error;

	/// Attempts to write an entire buffer `buf` to the writer.
	///
	/// This function returns a future that must be awaited for any work to
	/// happen.
	fn write_all(
		&mut self,
		buf: impl Buf,
	) -> impl Future<Output = Result<(), Self::Error>>;

	/// Flush the writer ensuring all bytes reach their destination.
	///
	/// This function returns a future that must be awaited for any work to
	/// happen.
	fn flush(
		&mut self,
	) -> impl Future<Output = Result<(), Self::Error>>;
}

impl<T> AsyncWrite for &mut T
where
	T: AsyncWrite + ?Sized,
{
	type Error = T::Error;

	fn write_all(
		&mut self,
		buf: impl Buf,
	) -> impl Future<Output = Result<(), Self::Error>> {
		(**self).write_all(buf)
	}

	fn flush(
		&mut self,
	) -> impl Future<Output = Result<(), Self::Error>> {
		(**self).flush()
	}
}

/// Types that can be converted to [`AsyncWrite`]ers.
pub trait IntoAsyncWriter<T: AsyncWrite> {
	/// Convert this type into an asynchronous writer `T`.
	fn into_async_writer(self) -> T;
}

impl<T: AsyncWrite> IntoAsyncWriter<T> for T {
	fn into_async_writer(self) -> T {
		self
	}
}
