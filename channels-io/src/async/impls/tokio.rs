use core::future::Future;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
	AsyncRead, AsyncWrite, Buf, BufMut, IntoAsyncReader,
	IntoAsyncWriter,
};

crate::util::newtype! {
	/// IO wrapper for the [`mod@tokio`] traits.
	TokioIo for:
}

impl<T> IntoAsyncWriter<TokioIo<T>> for T
where
	T: tokio::io::AsyncWrite + Unpin,
{
	fn into_async_writer(self) -> TokioIo<T> {
		TokioIo(self)
	}
}

impl<T> AsyncWrite for TokioIo<T>
where
	T: tokio::io::AsyncWrite + Unpin,
{
	type Error = tokio::io::Error;

	async fn write_all(
		&mut self,
		mut buf: impl Buf,
	) -> Result<(), Self::Error> {
		use tokio::io::ErrorKind as E;

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

impl<T> IntoAsyncReader<TokioIo<T>> for T
where
	T: tokio::io::AsyncRead + Unpin,
{
	fn into_async_reader(self) -> TokioIo<T> {
		TokioIo(self)
	}
}

impl<T> AsyncRead for TokioIo<T>
where
	T: tokio::io::AsyncRead + Unpin,
{
	type Error = tokio::io::Error;

	async fn read_all(
		&mut self,
		mut buf: impl BufMut,
	) -> Result<(), Self::Error> {
		use tokio::io::ErrorKind as E;

		while buf.has_remaining() {
			match self.0.read(buf.unfilled_mut()).await {
				Ok(0) => return Err(E::UnexpectedEof.into()),
				Ok(n) => buf.advance(n),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => return Err(e),
			}
		}

		Ok(())
	}
}
