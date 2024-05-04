use super::prelude::*;

newtype! {
	/// Wrapper IO type for [`futures::AsyncRead`] and [`futures::AsyncWrite`].
	Futures
}

impl_newtype_read! { Futures: ::futures::io::AsyncRead + Unpin }

impl<T> AsyncRead for Futures<T>
where
	T: ::futures::io::AsyncRead + Unpin,
{
	type Error = ::futures::io::Error;

	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut ReadBuf,
	) -> Poll<Result<(), Self::Error>> {
		use ::std::io::ErrorKind as E;

		while !buf.unfilled().is_empty() {
			match ready!(Pin::new(&mut self.0)
				.poll_read(cx, buf.unfilled_mut()))
			{
				Ok(0) => break,
				Ok(n) => buf.advance(n),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) => return Ready(Err(e)),
			}
		}

		Ready(Ok(()))
	}
}

impl_newtype_write! { Futures: ::futures::io::AsyncWrite + Unpin }

impl<T> AsyncWrite for Futures<T>
where
	T: ::futures::io::AsyncWrite + Unpin,
{
	type Error = ::futures::io::Error;

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
