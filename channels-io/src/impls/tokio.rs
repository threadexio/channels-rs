use super::prelude::*;

use ::std::io::ErrorKind as E;

newtype! {
	/// Wrapper IO type for [`tokio::io::AsyncRead`] and [`tokio::io::AsyncWrite`].
	Tokio
}

impl_newtype_read! { Tokio: ::tokio::io::AsyncRead + Unpin }

impl<T> AsyncRead for Tokio<T>
where
	T: ::tokio::io::AsyncRead + Unpin,
{
	type Error = ::tokio::io::Error;

	fn poll_read_slice(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut [u8],
	) -> Poll<Result<usize, Self::Error>> {
		let mut read_buf = ::tokio::io::ReadBuf::new(buf);
		ready!(Pin::new(&mut self.0).poll_read(cx, &mut read_buf))?;
		let n = read_buf.filled().len();
		Poll::Ready(Ok(n))
	}
}

impl_newtype_write! { Tokio: ::tokio::io::AsyncWrite  + Unpin }

impl<T> AsyncWrite for Tokio<T>
where
	T: ::tokio::io::AsyncWrite + Unpin,
{
	type Error = ::tokio::io::Error;

	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut WriteBuf,
	) -> Poll<Result<(), Self::Error>> {
		while !buf.remaining().is_empty() {
			match ready!(
				Pin::new(&mut self.0).poll_write(cx, buf.remaining())
			) {
				Ok(0) => break,
				Ok(n) => buf.advance(n),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => return Ready(Err(e)),
			}
		}

		Ready(Ok(()))
	}

	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		loop {
			match ready!(Pin::new(&mut self.0).poll_flush(cx)) {
				Ok(()) => return Ready(Ok(())),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => return Ready(Err(e)),
			}
		}
	}
}
