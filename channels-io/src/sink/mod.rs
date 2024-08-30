//! TODO: docs

use core::pin::Pin;
use core::task::{Context, Poll};

mod send;
use self::send::Send;

/// TODO: docs
pub trait Sink {
	/// TODO: docs
	type Item: ?Sized;

	/// TODO: docs
	type Error;

	/// TODO: docs
	fn send(&mut self, item: &Self::Item) -> Result<(), Self::Error>;
}

/// TODO: docs
pub trait AsyncSink {
	/// TODO: docs
	type Item: ?Sized;

	/// TODO: docs
	type Error;

	/// TODO: docs
	fn start_send(
		self: Pin<&mut Self>,
		item: &Self::Item,
	) -> Result<(), Self::Error>;

	/// TODO: docs
	fn poll_send(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>>;

	/// TODO: docs
	fn send<'a>(&'a mut self, item: &'a Self::Item) -> Send<'a, Self>
	where
		Self: Unpin,
		Self::Item: Unpin,
	{
		Send::new(self, item)
	}
}
