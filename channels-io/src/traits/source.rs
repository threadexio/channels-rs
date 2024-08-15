use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub trait Source {
	type Item;

	fn next(&mut self) -> Self::Item;
}

pub trait AsyncSource {
	type Item;

	fn poll_next(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Self::Item>;

	fn next(&mut self) -> Next<'_, Self>
	where
		Self: Unpin,
	{
		next(self)
	}
}

fn next<S>(source: &mut S) -> Next<'_, S>
where
	S: AsyncSource + Unpin + ?Sized,
{
	Next { source }
}

pub struct Next<'a, S>
where
	S: AsyncSource + Unpin + ?Sized,
{
	source: &'a mut S,
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
