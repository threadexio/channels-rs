use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::AsyncSource;

#[allow(missing_debug_implementations)]
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
	pub fn new(source: &'a mut S) -> Self {
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
		let Self { ref mut source } = *self;
		Pin::new(&mut **source).poll_next(cx)
	}
}
