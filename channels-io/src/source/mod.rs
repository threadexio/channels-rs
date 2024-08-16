//! TODO: docs

use core::pin::Pin;
use core::task::{Context, Poll};

mod next;
use self::next::Next;

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
		Next::new(self)
	}
}
