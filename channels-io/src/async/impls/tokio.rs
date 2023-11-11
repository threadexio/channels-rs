use core::pin::Pin;
use core::task::ready;
use core::task::{Context, Poll};

use crate::{
	AsyncRead, AsyncWrite, Buf, BufMut, IntoAsyncReader,
	IntoAsyncWriter,
};

crate::util::newtype! {
	/// IO wrapper for the [`mod@tokio`] traits.
	TokioIo for:
}

impl<T> IntoAsyncWriter<TokioIo<T>> for T
where
	T: tokio::io::AsyncWrite + Unpin,
{
	fn into_async_writer(self) -> TokioIo<T> {
		TokioIo(self)
	}
}

impl<T> AsyncWrite for TokioIo<T>
where
	T: tokio::io::AsyncWrite + Unpin,
{
	type Error = tokio::io::Error;

	fn poll_write_all(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		mut buf: impl Buf,
	) -> Poll<Result<(), Self::Error>> {
		use tokio::io::ErrorKind as E;

		while buf.has_remaining() {
			match ready!(
				Pin::new(&mut self.0).poll_write(cx, buf.unfilled())
			) {
				Ok(0) => {
					return Poll::Ready(Err(E::WriteZero.into()))
				},
				Ok(n) => buf.advance(n),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) if e.kind() == E::WouldBlock => {
					return Poll::Pending
				},
				Err(e) => return Poll::Ready(Err(e)),
			}
		}

		Poll::Ready(Ok(()))
	}

	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		Pin::new(&mut self.0).poll_flush(cx)
	}
}

impl<T> IntoAsyncReader<TokioIo<T>> for T
where
	T: tokio::io::AsyncRead + Unpin,
{
	fn into_async_reader(self) -> TokioIo<T> {
		TokioIo(self)
	}
}

impl<T> AsyncRead for TokioIo<T>
where
	T: tokio::io::AsyncRead + Unpin,
{
	type Error = tokio::io::Error;

	fn poll_read_all(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		mut buf: impl BufMut,
	) -> Poll<Result<(), Self::Error>> {
		use tokio::io::ErrorKind as E;

		while buf.has_remaining() {
			let mut read_buf =
				tokio::io::ReadBuf::new(buf.unfilled_mut());

			let (res, delta) =
				delta_filled_len(&mut read_buf, |buf| {
					Pin::new(&mut self.0).poll_read(cx, buf)
				});

			match ready!(res).map(|_| delta) {
				Ok(0) => {
					return Poll::Ready(Err(E::UnexpectedEof.into()))
				},
				Ok(n) => buf.advance(n),
				Err(e) if e.kind() == E::Interrupted => continue,
				Err(e) if e.kind() == E::WouldBlock => {
					return Poll::Pending
				},
				Err(e) => return Poll::Ready(Err(e)),
			}
		}

		Poll::Ready(Ok(()))
	}
}

fn delta_filled_len<F, T>(
	buf: &mut tokio::io::ReadBuf,
	f: F,
) -> (T, usize)
where
	F: FnOnce(&mut tokio::io::ReadBuf) -> T,
{
	let l0 = buf.filled().len();
	let ret = f(buf);
	let l1 = buf.filled().len();

	(ret, l1 - l0)
}
