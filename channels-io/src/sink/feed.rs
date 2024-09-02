use core::future::Future;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use super::AsyncSink;

/// Future for the [`feed()`] method.
///
/// [`feed()`]: super::AsyncSinkExt::feed
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct Feed<'a, S>
where
	S: AsyncSink + Unpin + ?Sized,
	S::Item: Unpin,
{
	sink: &'a mut S,
	item: Option<S::Item>,
}

impl<'a, S> Feed<'a, S>
where
	S: AsyncSink + Unpin + ?Sized,
	S::Item: Unpin,
{
	pub(crate) fn new(sink: &'a mut S, item: S::Item) -> Self {
		Self { sink, item: Some(item) }
	}
}

impl<'a, S> Future for Feed<'a, S>
where
	S: AsyncSink + Unpin + ?Sized,
	S::Item: Unpin,
{
	type Output = Result<(), S::Error>;

	fn poll(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		let this = self.get_mut();
		let mut sink = Pin::new(&mut *this.sink);

		assert!(
			this.item.is_some(),
			"Feed future polled after completion"
		);

		ready!(sink.as_mut().poll_ready(cx))?;

		// SAFETY: We checked this above.
		let item =
			this.item.take().expect("item should not have been None");
		sink.start_send(item)?;

		Poll::Ready(Ok(()))
	}
}
