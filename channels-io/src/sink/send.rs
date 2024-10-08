use core::future::Future;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use super::{AsyncSink, Sink};

pub fn send<S>(sink: &mut S, item: S::Item) -> Result<(), S::Error>
where
	S: Sink + ?Sized,
{
	sink.feed(item)?;
	sink.flush()
}

/// Future for the [`send()`] method.
///
/// [`send()`]: super::AsyncSinkExt::send
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct Send<'a, S>
where
	S: AsyncSink + Unpin + ?Sized,
	S::Item: Unpin,
{
	sink: &'a mut S,
	item: Option<S::Item>,
}

impl<'a, S> Send<'a, S>
where
	S: AsyncSink + Unpin + ?Sized,
	S::Item: Unpin,
{
	pub(crate) fn new(sink: &'a mut S, item: S::Item) -> Self {
		Self { sink, item: Some(item) }
	}
}

impl<'a, S> Future for Send<'a, S>
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

		if this.item.is_some() {
			ready!(sink.as_mut().poll_ready(cx))?;

			// SAFETY: We checked this above.
			let item = this
				.item
				.take()
				.expect("item should not have been None");

			sink.as_mut().start_send(item)?;
		}

		sink.poll_flush(cx)
	}
}
