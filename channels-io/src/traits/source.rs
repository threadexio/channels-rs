use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// TODO: docs
pub trait Source {
	/// TODO: docs
	type Item;

	/// TODO: docs
	fn next(&mut self) -> Self::Item;
}

/// TODO: docs
pub trait AsyncSource {
	/// TODO: docs
	type Item;

	/// TODO: docs
	fn poll_next(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Self::Item>;

	/// TODO: docs
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

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
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
