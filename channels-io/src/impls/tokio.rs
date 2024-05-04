use super::prelude::*;

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

	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut ReadBuf,
	) -> Poll<Result<(), Self::Error>> {
		use ::std::io::ErrorKind as E;

		while !buf.unfilled().is_empty() {
			let mut read_buf =
				tokio::io::ReadBuf::new(buf.unfilled_mut());

			match ready!(
				Pin::new(&mut self.0).poll_read(cx, &mut read_buf)
			) {
				Ok(()) => match read_buf.filled().len() {
					0 => break,
					n => buf.advance(n),
				},
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => return Ready(Err(e)),
			}
		}

		Ready(Ok(()))
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
		use ::std::io::ErrorKind as E;

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
		use ::std::io::ErrorKind as E;

		loop {
			match ready!(Pin::new(&mut self.0).poll_flush(cx)) {
				Ok(()) => return Ready(Ok(())),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => return Ready(Err(e)),
			}
		}
	}
}
