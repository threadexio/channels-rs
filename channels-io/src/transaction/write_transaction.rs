use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::{AsyncWrite, Write};

/// This trait provides the methods that are useful for write transactions that
/// work synchronously.
pub trait WriteTransaction: Write {
	/// Finish the transaction.
	fn finish(self) -> Result<(), Self::Error>
	where
		Self: Sized;
}

/// This trait provides the methods that are useful for write transactions that
/// work asynchronously.
pub trait AsyncWriteTransaction: AsyncWrite {
	/// Poll the future that finishes the transaction.
	fn poll_finish(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>>;

	/// Finish the transaction.
	fn finish(self) -> Finish<Self>
	where
		Self: Sized + Unpin,
	{
		Finish::new(self)
	}
}

#[allow(missing_debug_implementations)]
#[must_use = "futures do nothing unless you `.await` them"]
pub struct Finish<T>
where
	T: AsyncWriteTransaction + Unpin,
{
	transaction: T,
}

impl<T> Finish<T>
where
	T: AsyncWriteTransaction + Unpin,
{
	pub fn new(transaction: T) -> Self {
		Self { transaction }
	}
}

impl<T> Future for Finish<T>
where
	T: AsyncWriteTransaction + Unpin,
{
	type Output = Result<(), T::Error>;

	fn poll(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		let this = &mut self.transaction;
		Pin::new(this).poll_finish(cx)
	}
}
