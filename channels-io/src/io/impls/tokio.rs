use super::prelude::*;

use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

newtype! { Tokio }

impl_newtype_read! { Tokio: ::tokio::io::AsyncRead + Unpin + Send }

impl<T> AsyncRead for Tokio<T>
where
	T: ::tokio::io::AsyncRead + Unpin + Send,
{
	type Error = ::tokio::io::Error;

	async fn read<B>(&mut self, mut buf: B) -> Result<(), Self::Error>
	where
		B: ContiguousMut,
	{
		while buf.has_remaining_mut() {
			use ::tokio::io::ErrorKind as E;
			match self.0.read(buf.chunk_mut()).await {
				Ok(i) => buf.advance_mut(i),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => return Err(e),
			}
		}

		Ok(())
	}
}

impl_newtype_write! { Tokio: ::tokio::io::AsyncWrite  + Unpin + Send }

impl<T> AsyncWrite for Tokio<T>
where
	T: ::tokio::io::AsyncWrite + Unpin + Send,
{
	type Error = ::tokio::io::Error;

	async fn write<B>(
		&mut self,
		mut buf: B,
	) -> Result<(), Self::Error>
	where
		B: Contiguous,
	{
		while buf.has_remaining() {
			use ::tokio::io::ErrorKind as E;
			match self.0.write(buf.chunk()).await {
				Ok(i) => buf.advance(i),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => return Err(e),
			}
		}

		Ok(())
	}

	async fn flush(&mut self) -> Result<(), Self::Error> {
		loop {
			use ::tokio::io::ErrorKind as E;
			match self.0.flush().await {
				Ok(()) => break Ok(()),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => break Err(e),
			}
		}
	}
}
