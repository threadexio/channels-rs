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

	fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
		self.0.read(buf)
	}
}

impl_newtype_write! { Native: Write }

impl<T> Write for Native<T>
where
	T: Write,
{
	type Error = T::Error;

	fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
		self.0.write(buf)
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
		self.0.flush()
	}
}

newtype! {
	/// Wrapper IO type for [`AsyncRead`] and [`AsyncWrite`].
	NativeAsync
}

impl_newtype_read! { NativeAsync: AsyncRead }

impl<T> AsyncRead for NativeAsync<T>
where
	T: AsyncRead,
{
	type Error = T::Error;

	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut ReadBuf,
	) -> Poll<Result<(), Self::Error>> {
		Pin::new(&mut self.0).poll_read(cx, buf)
	}
}

impl_newtype_write! { NativeAsync: AsyncWrite }

impl<T> AsyncWrite for NativeAsync<T>
where
	T: AsyncWrite,
{
	type Error = T::Error;

	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut WriteBuf,
	) -> Poll<Result<(), Self::Error>> {
		Pin::new(&mut self.0).poll_write(cx, buf)
	}

	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		Pin::new(&mut self.0).poll_flush(cx)
	}
}
