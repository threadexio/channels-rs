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

/// Wrapper IO type for [`smol::io::AsyncRead`] and [`smol::io::AsyncWrite`].
///
/// [`smol::io::AsyncRead`]: ::smol::io::AsyncRead
/// [`smol::io::AsyncWrite`]: ::smol::io::AsyncWrite
#[derive(Debug)]
#[pin_project]
pub struct Smol<T>(#[pin] pub T);

impl_newtype! { Smol }

impl_newtype_read! { Smol: ::smol::io::AsyncRead }

impl<T> AsyncRead for Smol<T>
where
	T: ::smol::io::AsyncRead,
{
	type Error = ::smol::io::Error;

	fn poll_read_slice(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut [u8],
	) -> Poll<Result<usize, Self::Error>> {
		let this = self.project();
		this.0.poll_read(cx, buf)
	}
}

impl_newtype_write! { Smol: ::smol::io::AsyncWrite }

impl<T> AsyncWrite for Smol<T>
where
	T: ::smol::io::AsyncWrite,
{
	type Error = ::smol::io::Error;

	fn poll_write_slice(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, Self::Error>> {
		let this = self.project();
		this.0.poll_write(cx, buf)
	}

	fn poll_flush_once(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		let this = self.project();
		this.0.poll_flush(cx)
	}
}
