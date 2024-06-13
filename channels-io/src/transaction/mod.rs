//! IO transactions.

use alloc::vec::Vec;

use channels_macros::replace;

use crate::{AsyncWrite, Write};

mod buffered;
mod unbuffered;

#[derive(Debug)]
enum WriteVariant<'a> {
	Buffered(buffered::Buffered<'a>),
	Unbuffered(unbuffered::Unbuffered),
}

impl<'a> WriteVariant<'a> {
	pub fn new(kind: WriteTransactionKind<'a>) -> Self {
		match kind {
			WriteTransactionKind::Buffered(buf) => {
				Self::Buffered(buffered::new(buf))
			},
			WriteTransactionKind::Unbuffered => {
				Self::Unbuffered(unbuffered::new())
			},
		}
	}

	replace! {
		replace: {
			[
				(async =>)
				(await =>)
				(Write => Write)
				(add => add_sync)
				(finish => finish_sync)
			]
			[
				(async => async)
				(await => .await)
				(Write => AsyncWrite)
				(add => add_async)
				(finish => finish_async)
			]
		}
		code: {

	pub async fn add<W: Write>(
		&mut self,
		writer: &mut W,
		buf: &[u8],
	) -> Result<(), W::Error> {
		match self {
			Self::Buffered(x) => x.add(writer, buf) await,
			Self::Unbuffered(x) => x.add(writer, buf) await,
		}
	}

	pub async fn finish<W: Write>(
		self,
		writer: &mut W,
	) -> Result<(), W::Error> {
		match self {
			Self::Buffered(x) => x.finish(writer) await,
			Self::Unbuffered(x) => x.finish(writer) await,
		}
	}

		}
	}
}

/// Write transaction kind.
#[derive(Debug)]
pub enum WriteTransactionKind<'a> {
	/// A buffered write transaction will buffer data from [`add()`] in the
	/// provided buffer. When [`finish()`] is called, the transaction will write
	/// the entire buffer to the writer all at once.
	///
	/// [`add()`]: fn@WriteTransaction::add
	/// [`finish()`]: fn@WriteTransaction::finish
	Buffered(&'a mut Vec<u8>),
	/// An unbuffered write transaction will immediately write all data [`add()`]ed
	/// to it to the writer. It does no buffering and is equivalent ([mostly])
	/// to writing the data directly to the writer sequentially. For this kind of
	/// transaction [`finish()`] is a no-op.
	///
	/// [`add()`]:fn@WriteTransaction::add
	/// [`finish()`]: fn@WriteTransaction::finish
	/// [mostly]: struct@WriteTransaction
	Unbuffered,
}

replace! {
	replace: {
		[
			(async =>)
			(await =>)
			(Write => Write)
			(WriteTransaction => WriteTransaction)
			(_add => add_sync)
			(_finish => finish_sync)
			(cfg_sync => all())
			(cfg_async => any())
		]
		[
			(async => async)
			(await => .await)
			(Write => AsyncWrite)
			(WriteTransaction => AsyncWriteTransaction)
			(_add => add_async)
			(_finish => finish_async)
			(cfg_sync => any())
			(cfg_async => all())
		]
	}
	code: {

/// A write transaction is a way to abstract over how data is written to a writer.
///
/// There are 2 kinds of write transactions; "buffered" and "unbuffered". As the
/// name implies, the former does buffers data, while the latter does not. For
/// more details see: [`WriteTransactionKind`].
///
/// Transactions also conditionally choose to do IO based on if the last IO operation
/// succeeded. For example; an unbuffered transaction will immediately try to write
/// the provided data with each call to [`add()`]. If this write fails inside
/// the [`add()`] call, then the error is stored in the transaction and further
/// calls to [`add()`] will be no-ops. The stored error will be returned to the
/// user of transaction when [`finish()`]ing it. This is why [`finish()`] must
/// **always** be called. This pattern allows error handling to be centralized
/// and keep signatures of function that write data clean from [`Result<..., ...>`].
///
/// # Example
///
/// ```rust,no_run
#[cfg_attr(cfg_sync, doc = r#"
use std::fs::File;

use channels_io::{IntoWrite, Write};
use channels_io::transaction::{WriteTransaction, WriteTransactionKind};

fn write_hello<W: Write>(transaction: &mut WriteTransaction<W>) {
    transaction.add(b"Hello").add(b", ").add(b"world!\n");
}

fn write_foo<W: Write>(
    transaction: &mut WriteTransaction<W>,
    with_bar: bool,
) {
    transaction.add(b"Foo");
    if with_bar {
        transaction.add(b" & Bar");
    }
    transaction.add(b"\n");
}

fn my_fn() {
    let file = File::options().write(true).open("my_file").unwrap();
    let writer = file.into_write();

    let mut buf = Vec::with_capacity(512);
    let mut transaction =
         writer.transaction(WriteTransactionKind::Buffered(&mut buf));

    write_hello(&mut transaction);
    write_foo(&mut transaction, false);

    transaction.finish().unwrap();
}
"#)]
#[cfg_attr(cfg_async, doc = r#"
use tokio::fs::File;

use channels_io::{IntoWrite, AsyncWrite};
use channels_io::transaction::{AsyncWriteTransaction, WriteTransactionKind};

async fn write_hello<W: AsyncWrite>(transaction: &mut AsyncWriteTransaction<'_, W>) {
    transaction
        .add(b"Hello").await
        .add(b", ").await
        .add(b"world!\n").await;
}

async fn write_foo<W: AsyncWrite>(
    transaction: &mut AsyncWriteTransaction<'_, W>,
    with_bar: bool,
) {
    transaction.add(b"Foo").await;
    if with_bar {
        transaction.add(b" & Bar").await;
    }
    transaction.add(b"\n").await;
}

async fn my_fn() {
    let file = File::options().write(true).open("my_file").await.unwrap();
    let writer = file.into_write();

    let mut buf = Vec::with_capacity(512);
    let mut transaction =
        writer.transaction(WriteTransactionKind::Buffered(&mut buf));

    write_hello(&mut transaction).await;
    write_foo(&mut transaction, false).await;

    transaction.finish().await.unwrap();
}
"#)]
/// ```
///
/// [`WriteTransactionKind`]: enum@WriteTransactionKind
/// [`add()`]: fn@Self::add
/// [`finish()`]: fn@Self::finish
/// [`Result<..., ...>`]: enum@Result
#[derive(Debug)]
#[must_use = "transactions should always be `.finish()`ed"]
pub struct WriteTransaction<'a, W: Write> {
	writer: W,
	variant: WriteVariant<'a>,
	result: Result<(), W::Error>,
}

impl<'a, W: Write> WriteTransaction<'a, W> {
	/// Create a new transaction that buffers data in `buf`.
	///
	/// This is method is a shorthand for [`new()`].
	///
	/// [`new()`]: fn@Self::new
	#[inline]
	pub fn buffered(writer: W, buf: &'a mut Vec<u8>) -> Self {
		Self::new(writer, WriteTransactionKind::Buffered(buf))
	}

	/// Create a new transaction that does no buffering.
	///
	/// This is method is a shorthand for [`new()`].
	///
	/// [`new()`]: fn@Self::new
	#[inline]
	pub fn unbuffered(writer: W) -> Self {
		Self::new(writer, WriteTransactionKind::Unbuffered)
	}

	/// Create a new transaction of `kind`.
	///
	/// See: [`WriteTransactionKind`].
	///
	/// [`WriteTransactionKind`]: enum@WriteTransactionKind
	#[inline]
	pub fn new(writer: W, kind: WriteTransactionKind<'a>) -> Self {
		Self {
			writer, variant: WriteVariant::new(kind), result: Ok(())
		}
	}

	/// Get a reference to the underlying writer.
	#[inline]
	#[must_use]
	pub fn writer(&self) -> &W {
		&self.writer
	}

	/// Get a mutable reference to the underlying writer.
	#[inline]
	#[must_use]
	pub fn writer_mut(&mut self) -> &mut W {
		&mut self.writer
	}

	/// Get the error of the last IO operation that failed.
	///
	/// This method will return [`Some(...)`] with the error of the IO operation
	/// that failed. If all IO operations have succeeded, this method will return
	/// [`None`]. From the moment this method returns [`Some(...)`], all further
	/// calls to [`add()`] will be no-ops.
	///
	/// [`Some(...)`]: Option::Some
	/// [`None`]: Option::None
	/// [`add()`]: fn@Self::add
	pub fn last_error(&self) -> Option<&W::Error> {
		self.result.as_ref().err()
	}

	/// Ignore the failure of the last IO operation.
	///
	/// This method should be used with extreme caution. It discards any previous
	/// IO failure and allows the transaction to continue as if nothing was wrong.
	/// If it used without proper caution, this will silently cause the writer
	/// to enter an invalid state where some data [`add()`]ed to the transaction
	/// never gets written to the writer. This can happen in the following scenario:
	///
	/// 1) An IO failure occurs when adding data to the transaction.
	/// 2) Caller keeps adding data to the transaction. (which is never added
	///    because all calls to [`add()`] after an error are no-ops).
	/// 3) Caller calls this method to ignore the previous error.
	/// 4) Caller adds more data to the transaction.
	///
	/// In the above scenario the writer is left in a state where the data that
	/// was added after the error occurred but before it was ignored is never
	/// written to the writer.
	///
	/// You shouldn't generally have to use this method as ignoring a write
	/// failure and writing more data usually results in the same write failure.
	/// It only exists for some edge cases where some errors do not actually mean
	/// that the writer is unable to accept more data but are still treated as
	/// such.
	///
	/// [`add()`]: fn@Self::add
	pub  fn ignore_last_error(&mut self) {
		self.result = Ok(());
	}

	/// Add `buf` to the transaction.
	///
	/// It is up to the transaction kind to decide when `buf` should be written
	/// to the writer. For more see: [`WriteTransactionKind`]. It is guaranteed
	/// that when [`finish()`] returns, all data given to the transaction will
	/// be written to the writer.
	///
	/// [`WriteTransactionKind`]: enum@WriteTransactionKind
	/// [`finish()`]: fn@Self::finish
	pub async fn add(&mut self, buf: &[u8]) -> &mut Self {
		if self.result.is_ok() {
			self.result =
				self.variant._add(&mut self.writer, buf) await;
		}
		self
	}

	/// Finish the transaction.
	///
	/// It is up to the transaction kind whether this method does any work. For
	/// more see: [`WriteTransactionKind`].
	///
	/// Note that if a transaction is not finished, then it is not guaranteed that
	/// either all or any data at all has been written to the writer. It is also
	/// important that the result of this method is checked. An [`Err(...)`] variant
	/// in from this method means that writing to the writer has failed. This could
	/// either have happened at some previous point in a call to [`add()`] or
	/// in this method. Either way, the writer is left in an invalid state and
	/// it is not known if or how much data was written to the writer.
	///
	/// [`WriteTransactionKind`]: enum@WriteTransactionKind
	/// [`Err(...)`]: Result::Err
	/// [`add()`]: fn@Self::add
	#[must_use = "unchecked transaction result"]
	pub async fn finish(mut self) -> Result<(), W::Error> {
		match self.result {
			Ok(()) => self.variant._finish(&mut self.writer) await,
			r @ Err(_) => r,
		}
	}
}

	}
}
