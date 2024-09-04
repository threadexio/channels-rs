use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::AsyncSink;

/// Future for the [`flush()`] method.
///
/// [`flush()`]: super::AsyncSinkExt::flush
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct Flush<'a, S>
where
	S: AsyncSink + Unpin + ?Sized,
{
	sink: &'a mut S,
}

impl<'a, S> Flush<'a, S>
where
	S: AsyncSink + Unpin + ?Sized,
{
	pub(crate) fn new(sink: &'a mut S) -> Self {
		Self { sink }
	}
}

impl<'a, S> Future for Flush<'a, S>
where
	S: AsyncSink + Unpin + ?Sized,
{
	type Output = Result<(), S::Error>;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		Pin::new(&mut *self.sink).poll_flush(cx)
	}
}