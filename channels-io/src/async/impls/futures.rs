use core::future::Future;

use futures::{AsyncReadExt, AsyncWriteExt};

use crate::{
	AsyncRead, AsyncWrite, Buf, BufMut, IntoAsyncReader,
	IntoAsyncWriter,
};

crate::util::newtype! {
	/// IO wrapper for the [`mod@futures`] traits.
	FuturesIo for:
}

impl<T> IntoAsyncReader<FuturesIo<T>> for T
where
	T: futures::AsyncRead + Unpin,
{
	fn into_async_reader(self) -> FuturesIo<T> {
		FuturesIo(self)
	}
}

impl<T> AsyncRead for FuturesIo<T>
where
	T: futures::AsyncRead + Unpin,
{
	type Error = futures::io::Error;

	async fn read_all(
		&mut self,
		mut buf: impl BufMut,
	) -> Result<(), Self::Error> {
		use futures::io::ErrorKind as E;

		while buf.has_remaining() {
			match self.0.read(buf.unfilled_mut()).await {
				Ok(0) => return Err(E::WriteZero.into()),
				Ok(n) => buf.advance(n),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => return Err(e),
			}
		}

		Ok(())
	}
}

impl<T> IntoAsyncWriter<FuturesIo<T>> for T
where
	T: futures::AsyncWrite + Unpin,
{
	fn into_async_writer(self) -> FuturesIo<T> {
		FuturesIo(self)
	}
}

impl<T> AsyncWrite for FuturesIo<T>
where
	T: futures::AsyncWrite + Unpin,
{
	type Error = futures::io::Error;

	async fn write_all(
		&mut self,
		mut buf: impl Buf,
	) -> Result<(), Self::Error> {
		use futures::io::ErrorKind as E;

		while buf.has_remaining() {
			match self.0.write(buf.unfilled()).await {
				Ok(0) => return Err(E::WriteZero.into()),
				Ok(n) => buf.advance(n),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) if e.kind() == E::WouldBlock => {
					panic!("async io operation returned `WouldBlock`")
				},
				Err(e) => return Err(e),
			}
		}

		Ok(())
	}

	fn flush(
		&mut self,
	) -> impl Future<Output = Result<(), Self::Error>> {
		self.0.flush()
	}
}
