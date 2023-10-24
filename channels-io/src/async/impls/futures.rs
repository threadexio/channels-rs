use core::marker::Unpin;
use core::pin::Pin;
use core::task::ready;
use core::task::{Context, Poll};

use crate::util::newtype;
use crate::{
	AsyncRead, AsyncWrite, Buf, BufMut, IntoAsyncReader,
	IntoAsyncWriter,
};

newtype! { FuturesRead for: futures::AsyncRead + Unpin }

impl<T> IntoAsyncReader<FuturesRead<T>> for T
where
	T: futures::AsyncRead + Unpin,
{
	fn into_async_reader(self) -> FuturesRead<T> {
		FuturesRead(self)
	}
}

impl<T> AsyncRead for FuturesRead<T>
where
	T: futures::AsyncRead + Unpin,
{
	type Error = futures::io::Error;

	fn poll_read_all(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		mut buf: impl BufMut,
	) -> Poll<Result<(), Self::Error>> {
		use futures::io::ErrorKind as E;

		while !buf.has_remaining_mut() {
			match ready!(Pin::new(&mut self.0)
				.poll_read(cx, buf.unfilled_mut()))
			{
				Ok(0) => {
					return Poll::Ready(Err(E::WriteZero.into()))
				},
				Ok(n) => buf.advance_mut(n),
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

newtype! { FuturesWrite for: futures::AsyncWrite + Unpin }

impl<T> IntoAsyncWriter<FuturesWrite<T>> for T
where
	T: futures::AsyncWrite + Unpin,
{
	fn into_async_writer(self) -> FuturesWrite<T> {
		FuturesWrite(self)
	}
}

impl<T> AsyncWrite for FuturesWrite<T>
where
	T: futures::AsyncWrite + Unpin,
{
	type Error = futures::io::Error;

	fn poll_write_all(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		mut buf: impl Buf,
	) -> Poll<Result<(), Self::Error>> {
		use futures::io::ErrorKind as E;

		while !buf.has_remaining() {
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
