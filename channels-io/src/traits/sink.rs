use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// TODO: docs
pub trait Sink {
	/// TODO: docs
	type Item;

	/// TODO: docs
	type Error;

	/// TODO: docs
	fn send(&mut self, item: Self::Item) -> Result<(), Self::Error>;
}

/// TODO: docs
pub trait AsyncSink {
	/// TODO: docs
	type Item;

	/// TODO: docs
	type Error;

	/// TODO: docs
	fn start_send(
		self: Pin<&mut Self>,
		item: Self::Item,
	) -> Result<(), Self::Error>;

	/// TODO: docs
	fn poll_send(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>>;

	/// TODO: docs
	fn send(&mut self, item: Self::Item) -> Send<'_, Self>
	where
		Self: Unpin,
		Self::Item: Unpin,
	{
		send(self, item)
	}
}

fn send<S>(sink: &mut S, item: S::Item) -> Send<'_, S>
where
	S: AsyncSink + Unpin + ?Sized,
	S::Item: Unpin,
{
	Send { sink, item: Some(item) }
}

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct Send<'a, S>
where
	S: AsyncSink + Unpin + ?Sized,
	S::Item: Unpin,
{
	sink: &'a mut S,
	item: Option<S::Item>,
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
