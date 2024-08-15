use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub trait Sink {
	type Item;

	type Error;

	fn send(&mut self, item: Self::Item) -> Result<(), Self::Error>;
}

pub trait AsyncSink {
	type Item;

	type Error;

	fn start_send(
		self: Pin<&mut Self>,
		item: Self::Item,
	) -> Result<(), Self::Error>;

	fn poll_send(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>>;

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
