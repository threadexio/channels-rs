//! [`Source`] and [`AsyncSource`] traits.

use core::pin::Pin;
use core::task::{Context, Poll};

use crate::util::assert_future;

mod next;

pub use self::next::Next;

/// This trait allows receiving items from somewhere.
///
/// Types implementing this trait are called "sources".
pub trait Source {
	/// The type of items the source receives.
	type Item;

	/// Get the next item.
	fn next(&mut self) -> Self::Item;

	/// Get an estimation of the number of items yet to be received.
	///
	/// Returns a tuple where the 2 elements are the lower and upper bounds of the number
	/// of items expected to be received. The upper bound is an `Option<usize>` to account
	/// for cases where the upper bound is not known. In such cases, implementations should
	/// return `None` as the upper bound.
	///
	/// The default implementation returns `(0, None)`.
	fn size_hint(&self) -> (usize, Option<usize>) {
		(0, None)
	}
}

/// Extension trait for [`Source`].
pub trait SourceExt: Source {}

impl<T: Source + ?Sized> SourceExt for T {}

/// The asynchronous version of [`Source`].
pub trait AsyncSource {
	/// The type of items the source receives.
	type Item;

	/// Attempt to receive the next item from the source.
	fn poll_next(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Self::Item>;

	/// Get an estimation of the number of items yet to be received.
	///
	/// Returns a tuple where the 2 elements are the lower and upper bounds of the number
	/// of items expected to be received. The upper bound is an `Option<usize>` to account
	/// for cases where the upper bound is not known. In such cases, implementations should
	/// return `None` as the upper bound.
	///
	/// The default implementation returns `(0, None)`.
	fn size_hint(&self) -> (usize, Option<usize>) {
		(0, None)
	}
}

/// Extension trait for [`AsyncSource`].
pub trait AsyncSourceExt: AsyncSource {
	/// The asynchronous version of [`next()`].
	///
	/// [`next()`]: Source::next
	fn next(&mut self) -> Next<'_, Self>
	where
		Self: Unpin,
	{
		assert_future::<Self::Item, _>(Next::new(self))
	}
}

impl<T: AsyncSource + ?Sized> AsyncSourceExt for T {}
