use core::pin::Pin;
use core::task::ready;
use core::task::{Context, Poll};

use crate::buf::{IoSliceMut, IoSliceRef};
use crate::util::newtype;
use crate::{
	AsyncRead, AsyncWrite, IntoAsyncReader, IntoAsyncWriter,
};

use tokio::io as tio;

newtype! { TokioAsyncWrite for: tio::AsyncWrite + Unpin }

impl<T> IntoAsyncWriter<TokioAsyncWrite<T>> for T
where
	T: tio::AsyncWrite + Unpin,
{
	fn into_async_writer(self) -> TokioAsyncWrite<T> {
		TokioAsyncWrite(self)
	}
}

impl<T> AsyncWrite for TokioAsyncWrite<T>
where
	T: tio::AsyncWrite + Unpin,
{
	type Error = std::io::Error;

	fn poll_write_all(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut IoSliceRef,
	) -> Poll<Result<(), Self::Error>> {
		use std::io::ErrorKind as E;

		while !buf.is_empty() {
			match ready!(Pin::new(&mut self.0).poll_write(cx, buf)) {
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

newtype! { TokioAsyncRead for: tio::AsyncRead + Unpin }

impl<T> IntoAsyncReader<TokioAsyncRead<T>> for T
where
	T: tio::AsyncRead + Unpin,
{
	fn into_async_reader(self) -> TokioAsyncRead<T> {
		TokioAsyncRead(self)
	}
}

impl<T> AsyncRead for TokioAsyncRead<T>
where
	T: tio::AsyncRead + Unpin,
{
	type Error = std::io::Error;

	fn poll_read_all(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut IoSliceMut,
	) -> Poll<Result<(), Self::Error>> {
		use std::io::ErrorKind as E;

		while !buf.is_empty() {
			let mut read_buf = tio::ReadBuf::new(buf);

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

fn delta_filled_len<F, T>(buf: &mut tio::ReadBuf, f: F) -> (T, usize)
where
	F: FnOnce(&mut tio::ReadBuf) -> T,
{
	let l0 = buf.filled().len();
	let ret = f(buf);
	let l1 = buf.filled().len();

	(ret, l1 - l0)
}
