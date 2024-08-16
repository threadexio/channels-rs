use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::AsyncWriteExt;

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct Flush<'a, T>
where
	T: AsyncWriteExt + Unpin + ?Sized,
{
	writer: &'a mut T,
}

impl<'a, T> Flush<'a, T>
where
	T: AsyncWriteExt + Unpin + ?Sized,
{
	pub fn new(writer: &'a mut T) -> Self {
		Self { writer }
	}
}

impl<'a, T> Future for Flush<'a, T>
where
	T: AsyncWriteExt + Unpin + ?Sized,
{
	type Output = Result<(), T::Error>;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		let Self { ref mut writer, .. } = *self;
		Pin::new(&mut **writer).poll_flush(cx)
	}
}
