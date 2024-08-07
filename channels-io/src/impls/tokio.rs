use super::prelude::*;

/// Wrapper IO type for [`tokio::io::AsyncRead`] and [`tokio::io::AsyncWrite`].
///
/// [`tokio::io::AsyncRead`]: ::tokio::io::AsyncRead
/// [`tokio::io::AsyncWrite`]: ::tokio::io::AsyncWrite
#[derive(Debug)]
#[pin_project]
pub struct Tokio<T>(#[pin] pub T);

impl_newtype! { Tokio }

impl_newtype_read! { Tokio: ::tokio::io::AsyncRead }

impl<T> AsyncRead for Tokio<T>
where
	T: ::tokio::io::AsyncRead,
{
	type Error = ::tokio::io::Error;

	fn poll_read_slice(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut [u8],
	) -> Poll<Result<usize, Self::Error>> {
		let this = self.project();

		let mut read_buf = ::tokio::io::ReadBuf::new(buf);
		ready!(this.0.poll_read(cx, &mut read_buf))?;
		let n = read_buf.filled().len();
		Poll::Ready(Ok(n))
	}
}

impl_newtype_write! { Tokio: ::tokio::io::AsyncWrite }

impl<T> AsyncWrite for Tokio<T>
where
	T: ::tokio::io::AsyncWrite,
{
	type Error = ::tokio::io::Error;

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
