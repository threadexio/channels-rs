use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::buf::Buf;

use super::AsyncWriteExt;

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct WriteBufAll<'a, T, B>
where
	T: AsyncWriteExt + Unpin + ?Sized,
	B: Buf + Unpin,
{
	writer: &'a mut T,
	buf: B,
}

impl<'a, T, B> WriteBufAll<'a, T, B>
where
	T: AsyncWriteExt + Unpin + ?Sized,
	B: Buf + Unpin,
{
	pub fn new(writer: &'a mut T, buf: B) -> Self {
		Self { writer, buf }
	}
}

impl<'a, T, B> Future for WriteBufAll<'a, T, B>
where
	T: AsyncWriteExt + Unpin + ?Sized,
	B: Buf + Unpin,
{
	type Output = Result<(), T::Error>;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		let Self { ref mut writer, ref mut buf, .. } = *self;
		Pin::new(&mut **writer).poll_write_buf_all(cx, buf)
	}
}
