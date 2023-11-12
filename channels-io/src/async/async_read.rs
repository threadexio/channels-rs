use core::future::Future;

use crate::BufMut;

/// This is the asynchronous version of [`Read`].
///
/// [`Read`]: crate::Read
pub trait AsyncRead {
	/// The error returned by [`AsyncRead::read_all()`].
	type Error;

	/// Read as many bytes an needed to fill `buf`.
	///
	/// This function returns a future that must be awaited for any work to
	/// happen.
	fn read_all(
		&mut self,
		buf: impl BufMut,
	) -> impl Future<Output = Result<(), Self::Error>>;
}

impl<T> AsyncRead for &mut T
where
	T: AsyncRead + ?Sized,
{
	type Error = T::Error;

	fn read_all(
		&mut self,
		buf: impl BufMut,
	) -> impl Future<Output = Result<(), Self::Error>> {
		(**self).read_all(buf)
	}
}

/// Types that can be converted to [`AsyncRead`]ers.
pub trait IntoAsyncReader<T: AsyncRead> {
	/// Convert this type into an asynchronous reader `T`.
	fn into_async_reader(self) -> T;
}

impl<T: AsyncRead> IntoAsyncReader<T> for T {
	fn into_async_reader(self) -> T {
		self
	}
}
