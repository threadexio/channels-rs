//! [`Sink`] and [`AsyncSink`] traits.

use core::pin::Pin;
use core::task::{Context, Poll};

use crate::util::assert_future;

mod feed;
mod flush;
mod ready;
mod send;

use self::send::send;

pub use self::feed::Feed;
pub use self::flush::Flush;
pub use self::ready::Ready;
pub use self::send::Send;

/// This trait allows sending items someplace.
///
/// Types implementing this trait are called "sinks".
pub trait Sink {
	/// The type of the items the sink accepts.
	type Item;

	/// Error type returned by the sink.
	type Error;

	/// Prepare the sink to ready to receive the next value.
	///
	/// Returns `Ok(())` if the sink is ready to receive the next item. If the sink cannot
	/// accept an item, `Err(...)` is returned.
	///
	/// This method **can optionally** block until the sink is ready.
	fn ready(&mut self) -> Result<(), Self::Error>;

	/// Feed the next item to the sink without flushing it.
	///
	/// Returns `Ok(())` to indicate a successful operation and `Err(...)` if the sink
	/// could not accept the item.
	///
	/// Each call to [`send()`] will also perform a flush of the sink. When sending many
	/// items, it might be advantageous to avoid these extra flushes and instead batch the
	/// items and flush the sink once. This method will send the item to the sink but it
	/// will not flush it. This allows flushing the sink at some later point with [`flush()`].
	///
	/// [`send()`]: SinkExt::send
	/// [`flush()`]: Sink::flush
	fn feed(&mut self, item: Self::Item) -> Result<(), Self::Error>;

	/// Flush the sink ensuring all pending items reach their destination.
	///
	/// Returns `Ok(())` to indicate a successful operation and `Err(...)` if one or
	/// more items could not be flushed.
	fn flush(&mut self) -> Result<(), Self::Error>;
}

/// Extension trait for [`Sink`].
pub trait SinkExt: Sink {
	/// Send an item to the sink.
	///
	/// Returns `Ok(())` if the item was successfully sent and `Err(...)` if not.
	///
	/// This method will also flush the sink afterwards. If items do not need to immediately
	/// reach their destination, prefer [`feed()`] and [`flush()`].
	///
	/// [`feed()`]: Sink::feed
	/// [`flush()`]: Sink::flush
	fn send(&mut self, item: Self::Item) -> Result<(), Self::Error> {
		send(self, item)
	}
}

impl<T: Sink + ?Sized> SinkExt for T {}

/// The asynchronous version of [`Sink`].
pub trait AsyncSink {
	/// The type of the items the sink accepts.
	type Item;

	/// Error type returned by the sink.
	type Error;

	/// Attempt to prepare the sink to receive an item.
	///
	/// Returns `Poll::Ready(Ok(()))` only when the sink is ready to receive the next item.
	/// If an error occurs and the sink is unable to receive an item, it returns
	/// `Poll::Ready(Err(...))`.
	fn poll_ready(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>>;

	/// Attempt to start the sending of an item.
	///
	/// Returns `Ok(())` if the item was put in the sink successfully. Otherwise it returns,
	/// `Err(...)`.
	///
	/// The sending of an item is completed by [`poll_flush`].
	///
	/// [`poll_flush`]: AsyncSink::poll_flush
	fn start_send(
		self: Pin<&mut Self>,
		item: Self::Item,
	) -> Result<(), Self::Error>;

	/// Attempt to flush all items in the sink.
	///
	/// Returns `Poll::Ready(Ok(()))` when the sink contains no buffered items. If the sink
	/// could not flush one or more buffered items, it returns `Poll::Ready(Err(...))`.
	fn poll_flush(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>>;
}

/// Extension trait for [`AsyncSink`].
pub trait AsyncSinkExt: AsyncSink {
	/// The asynchronous version of [`ready()`].
	///
	/// [`ready()`]: Sink::ready
	fn ready(&mut self) -> Ready<'_, Self>
	where
		Self: Unpin,
	{
		assert_future::<Result<(), Self::Error>, _>(Ready::new(self))
	}

	/// The asynchronous version of [`feed()`].
	///
	/// [`feed()`]: Sink::feed
	fn feed(&mut self, item: Self::Item) -> Feed<'_, Self>
	where
		Self: Unpin,
		Self::Item: Unpin,
	{
		assert_future::<Result<(), Self::Error>, _>(Feed::new(
			self, item,
		))
	}

	/// The asynchronous version of [`flush()`].
	///
	/// [`flush()`]: Sink::flush
	fn flush(&mut self) -> Flush<'_, Self>
	where
		Self: Unpin,
	{
		assert_future::<Result<(), Self::Error>, _>(Flush::new(self))
	}

	/// The asynchronous version of [`send()`].
	///
	/// [`send()`]: SinkExt::send
	fn send(&mut self, item: Self::Item) -> Send<'_, Self>
	where
		Self: Unpin,
		Self::Item: Unpin,
	{
		assert_future::<Result<(), Self::Error>, _>(Send::new(
			self, item,
		))
	}
}

impl<T: AsyncSink + ?Sized> AsyncSinkExt for T {}
