use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::AsyncSource;

/// Future for the [`next()`] method.
///
/// [`next()`]: super::AsyncSourceExt::next
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct Next<'a, S>
where
	S: AsyncSource + Unpin + ?Sized,
{
	source: &'a mut S,
}

impl<'a, S> Next<'a, S>
where
	S: AsyncSource + Unpin + ?Sized,
{
	pub(crate) fn new(source: &'a mut S) -> Self {
		Self { source }
	}
}

impl<'a, S> Future for Next<'a, S>
where
	S: AsyncSource + Unpin + ?Sized,
{
	type Output = S::Item;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		Pin::new(&mut *self.source).poll_next(cx)
	}
}
