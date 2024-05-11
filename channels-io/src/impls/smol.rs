use super::prelude::*;

#[cfg(not(feature = "std"))]
mod smol_error_impls {
	use crate::{IoError, ReadError, WriteError};
	use ::smol::io::ErrorKind as E;

	impl IoError for ::smol::io::Error {
		fn should_retry(&self) -> bool {
			self.kind() == E::Interrupted
		}
	}

	impl ReadError for ::smol::io::Error {
		fn eof() -> Self {
			Self::from(E::UnexpectedEof)
		}
	}

	impl WriteError for ::smol::io::Error {
		fn write_zero() -> Self {
			Self::from(E::WriteZero)
		}
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

	fn poll_write_slice(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, Self::Error>> {
		Pin::new(&mut self.0).poll_write(cx, buf)
	}

	fn poll_flush_once(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		Pin::new(&mut self.0).poll_flush(cx)
	}
}
