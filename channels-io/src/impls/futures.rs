use super::prelude::*;

/// Wrapper IO type for [`futures::AsyncRead`] and [`futures::AsyncWrite`].
///
/// [`futures::AsyncRead`]: ::futures::AsyncRead
/// [`futures::AsyncWrite`]:  ::futures::AsyncWrite
#[derive(Debug)]
#[pin_project]
pub struct Futures<T>(#[pin] pub T);

impl_newtype! { Futures }

impl_newtype_read! { Futures: ::futures::io::AsyncRead }

impl<T> AsyncRead for Futures<T>
where
	T: ::futures::io::AsyncRead,
{
	type Error = ::futures::io::Error;

	fn poll_read_slice(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut [u8],
	) -> Poll<Result<usize, Self::Error>> {
		let this = self.project();
		this.0.poll_read(cx, buf)
	}
}

impl_newtype_write! { Futures: ::futures::io::AsyncWrite }

impl<T> AsyncWrite for Futures<T>
where
	T: ::futures::io::AsyncWrite,
{
	type Error = ::futures::io::Error;

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
