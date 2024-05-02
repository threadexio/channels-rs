use super::prelude::*;

use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

newtype! {
	/// Wrapper IO type for [`tokio::io::AsyncRead`] and [`tokio::io::AsyncWrite`].
	Tokio
}

impl_newtype_read! { Tokio: ::tokio::io::AsyncRead + Unpin + Send }

impl<T> AsyncRead for Tokio<T>
where
	T: ::tokio::io::AsyncRead + Unpin + Send,
{
	type Error = ::tokio::io::Error;

	async fn read(
		&mut self,
		mut buf: &mut [u8],
	) -> Result<(), Self::Error> {
		while !buf.is_empty() {
			use ::tokio::io::ErrorKind as E;
			match self.0.read(buf).await {
				Ok(i) => buf = &mut buf[i..],
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

	async fn write(
		&mut self,
		mut buf: &[u8],
	) -> Result<(), Self::Error> {
		while !buf.is_empty() {
			use ::tokio::io::ErrorKind as E;
			match self.0.write(buf).await {
				Ok(i) => buf = &buf[i..],
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
