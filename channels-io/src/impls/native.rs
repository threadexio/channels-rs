use super::prelude::*;

newtype! {
	/// Wrapper IO type for [`Read`] and [`Write`].
	Native
}

impl_newtype_read! { Native: Read }

impl<T> Read for Native<T>
where
	T: Read,
{
	type Error = T::Error;

	fn read_slice(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, Self::Error> {
		self.0.read_slice(buf)
	}
}

impl_newtype_write! { Native: Write }

impl<T> Write for Native<T>
where
	T: Write,
{
	type Error = T::Error;

	fn write_slice(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		self.0.write_slice(buf)
	}

	fn flush_once(&mut self) -> Result<(), Self::Error> {
		self.0.flush_once()
	}
}

newtype! {
	/// Wrapper IO type for [`AsyncRead`] and [`AsyncWrite`].
	NativeAsync
}

impl_newtype_read! { NativeAsync: AsyncRead }

impl<T> AsyncRead for NativeAsync<T>
where
	T: AsyncRead + Unpin,
{
	type Error = T::Error;

	fn poll_read_slice(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut [u8],
	) -> Poll<Result<usize, Self::Error>> {
		Pin::new(&mut self.0).poll_read_slice(cx, buf)
	}
}

impl_newtype_write! { NativeAsync: AsyncWrite }

impl<T> AsyncWrite for NativeAsync<T>
where
	T: AsyncWrite + Unpin,
{
	type Error = T::Error;

	fn poll_write_slice(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, Self::Error>> {
		Pin::new(&mut self.0).poll_write_slice(cx, buf)
	}

	fn poll_flush_once(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		Pin::new(&mut self.0).poll_flush_once(cx)
	}
}
