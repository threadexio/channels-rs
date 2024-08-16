use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::buf::BufMut;

use super::AsyncReadExt;

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct ReadBuf<'a, T, B>
where
	T: AsyncReadExt + Unpin + ?Sized,
	B: BufMut + Unpin,
{
	reader: &'a mut T,
	buf: B,
}

impl<'a, T, B> ReadBuf<'a, T, B>
where
	T: AsyncReadExt + Unpin + ?Sized,
	B: BufMut + Unpin,
{
	pub fn new(reader: &'a mut T, buf: B) -> Self {
		Self { reader, buf }
	}
}

impl<'a, T, B> Future for ReadBuf<'a, T, B>
where
	T: AsyncReadExt + Unpin + ?Sized,
	B: BufMut + Unpin,
{
	type Output = Result<(), T::Error>;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Self::Output> {
		let Self { ref mut reader, ref mut buf } = *self;
		Pin::new(&mut **reader).poll_read_buf(cx, buf)
	}
}
