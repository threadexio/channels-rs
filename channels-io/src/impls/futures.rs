use super::prelude::*;

newtype! {
	/// Wrapper IO type for [`futures::AsyncRead`] and [`futures::AsyncWrite`].
	///
	/// [`futures::AsyncRead`]: ::futures::AsyncRead
	/// [`futures::AsyncWrite`]:  ::futures::AsyncWrite
	Futures
}

impl_newtype_read! { Futures: ::futures::io::AsyncRead + Unpin }

impl<T> AsyncRead for Futures<T>
where
	T: ::futures::io::AsyncRead + Unpin,
{
	type Error = ::futures::io::Error;

	fn poll_read_slice(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut [u8],
	) -> Poll<Result<usize, Self::Error>> {
		Pin::new(&mut self.0).poll_read(cx, buf)
	}
}

impl_newtype_write! { Futures: ::futures::io::AsyncWrite + Unpin }

impl<T> AsyncWrite for Futures<T>
where
	T: ::futures::io::AsyncWrite + Unpin,
{
	type Error = ::futures::io::Error;

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
