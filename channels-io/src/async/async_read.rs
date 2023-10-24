use core::future::Future;
use core::marker::Unpin;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::decouple;
use crate::BufMut;

/// This is the asynchronous version of [`Read`].
///
/// [`Read`]: crate::Read
pub trait AsyncRead {
	/// The error returned by [`AsyncRead::read_all()`].
	type Error;

	/// Poll the underlying reader and read some data into `buf` if possible.
	fn poll_read_all(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: impl BufMut,
	) -> Poll<Result<(), Self::Error>>;

	/// Read as many bytes an needed to fill `buf`.
	///
	/// This function returns a future that must be awaited for any work to
	/// happen.
	fn read_all<B: BufMut + Unpin>(
		&mut self,
		buf: B,
	) -> ReadAll<'_, B, Self>
	where
		Self: Unpin,
	{
		ReadAll::new(self, buf)
	}
}

impl<T> AsyncRead for &mut T
where
	T: AsyncRead + Unpin + ?Sized,
{
	type Error = T::Error;

	fn poll_read_all(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: impl BufMut,
	) -> Poll<Result<(), Self::Error>> {
		Pin::new(&mut **self).poll_read_all(cx, buf)
	}
}

#[doc(hidden)]
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct ReadAll<'a, B, R>
where
	B: BufMut + Unpin,
	R: AsyncRead + Unpin + ?Sized,
{
	reader: &'a mut R,
	buf: B,
}

impl<'a, B, R> ReadAll<'a, B, R>
where
	B: BufMut + Unpin,
	R: AsyncRead + Unpin + ?Sized,
{
	pub(self) fn new(reader: &'a mut R, buf: B) -> Self {
		Self { reader, buf }
	}
}

impl<'a, B, R> Future for ReadAll<'a, B, R>
where
	B: BufMut + Unpin,
	R: AsyncRead + Unpin + ?Sized,
{
	type Output = Result<(), R::Error>;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Self::Output> {
		let buf = decouple!(self.buf; as mut);
		Pin::new(&mut *self.reader).poll_read_all(cx, buf)
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
