use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::AsyncSink;

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct Send<'a, S>
where
	S: AsyncSink + Unpin + ?Sized,
	S::Item: Unpin,
{
	sink: &'a mut S,
	item: Option<&'a S::Item>,
}

impl<'a, S> Send<'a, S>
where
	S: AsyncSink + Unpin + ?Sized,
	S::Item: Unpin,
{
	pub fn new(sink: &'a mut S, item: &'a S::Item) -> Self {
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
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		let Self { ref mut sink, ref mut item } = *self;
		let mut sink = Pin::new(&mut **sink);

		if let Some(item) = item.take() {
			sink.as_mut().start_send(item)?;
		}

		sink.poll_send(cx)
	}
}
