use super::prelude::*;

#[allow(unused_imports)]
use ::smol::io::ErrorKind as E;

#[cfg(not(feature = "std"))]
impl ReadError for ::smol::io::Error {
	fn eof() -> Self {
		Self::from(E::UnexpectedEof)
	}

	fn should_retry(&self) -> bool {
		self.kind() == E::Interrupted
	}
}

newtype! {
	/// Wrapper IO type for [`smol::io::AsyncRead`] and [`smol::io::AsyncWrite`].
	Smol
}

impl_newtype_read! { Smol: ::smol::io::AsyncRead + Unpin }

impl<T> AsyncRead for Smol<T>
where
	T: ::smol::io::AsyncRead + Unpin,
{
	type Error = ::smol::io::Error;

	fn poll_read_slice(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut [u8],
	) -> Poll<Result<usize, Self::Error>> {
		Pin::new(&mut self.0).poll_read(cx, buf)
	}
}

impl_newtype_write! { Smol: ::smol::io::AsyncWrite+ Unpin }

impl<T> AsyncWrite for Smol<T>
where
	T: ::smol::io::AsyncWrite + Unpin,
{
	type Error = ::smol::io::Error;

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