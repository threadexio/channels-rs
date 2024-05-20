#![allow(missing_docs)] // TODO: docs

use core::pin::Pin;
use core::task::{Context, Poll};

use alloc::vec::Vec;

use crate::{AsyncWrite, Write};

/// An abstraction that allows bunching up many [`write()`] calls into one.
///
/// This abstraction can hold a reference to a [`Vec<u8>`] in which it will
/// write data given to it from the [`Write`] and [`AsyncWrite`] it implements.
/// Upon calling [`finish_sync()`] or [`finish_async()`] it will write the contents
/// of that buffer to the underlying writer. It must also be noted that if configured
/// to do buffering, the [`WriteTransaction`] will also batch calls to [`flush()`]
/// and call it once, if needed, when finished. When a buffer is not provided, it
/// passes through the [`Write`] and [`AsyncWrite`] implementations to the underlying
/// writer in a "1-1" manner.
///
/// [`write()`]: fn@Write::write
/// [`flush()`]: fn@Write::flush
/// [`finish_sync()`]: fn@WriteTransaction::finish_sync
/// [`finish_async()`]: fn@WriteTransaction::finish_async
#[derive(Debug)]
#[must_use = "transactions should always be `.finish_*()`ed"]
pub struct WriteTransaction<'a, W: ?Sized> {
	writer: &'a mut W,
	buf: Option<&'a mut Vec<u8>>,
	wants_flush: bool,
}

impl<'a, W: ?Sized> WriteTransaction<'a, W> {
	/// Create a new transaction that uses `buf` to buffer up writes.
	///
	/// This method will also clear `buf`.
	pub fn buffered(writer: &'a mut W, buf: &'a mut Vec<u8>) -> Self {
		buf.clear();
		Self { writer, buf: Some(buf), wants_flush: false }
	}

	/// Create a new transaction that does no buffering.
	pub fn unbuffered(writer: &'a mut W) -> Self {
		Self { writer, buf: None, wants_flush: false }
	}

	/// Get a reference to the underlying writer.
	#[inline]
	#[must_use]
	pub fn writer(&self) -> &W {
		self.writer
	}

	/// Get a mutable reference to the underlying writer.
	#[inline]
	#[must_use]
	pub fn writer_mut(&mut self) -> &mut W {
		self.writer
	}
}

impl<'a, W: Write + ?Sized> Write for WriteTransaction<'a, W> {
	type Error = W::Error;

	fn write_slice(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		match self.buf.as_mut() {
			None => self.writer.write_slice(buf),
			Some(x) => {
				x.extend_from_slice(buf);
				Ok(buf.len())
			},
		}
	}

	fn flush_once(&mut self) -> Result<(), Self::Error> {
		match self.buf {
			None => self.writer.flush(),
			Some(_) => {
				self.wants_flush = true;
				Ok(())
			},
		}
	}
}

impl<'a, W: Write + ?Sized> WriteTransaction<'a, W> {
	/// Finish the transaction.
	///
	/// If the transaction does buffering then this method will attempt to write
	/// the whole buffer to the writer. Additionally, if [`flush()`] was ever
	/// called on the transaction, then this method will also attempt to flush
	/// the writer after writing to it. However, if the transaction is was created
	/// with [`unbuffered()`] then this method is simply a no-op. It is semantically
	/// correct that a [`WriteTransaction`] is always finished regardless of whether
	/// the finish operation does any work.
	///
	/// [`flush()`]: fn@Write::flush
	/// [`unbuffered()`]: fn@WriteTransaction::unbuffered
	pub fn finish_sync(self) -> Result<(), W::Error> {
		let Self { buf, wants_flush, writer } = self;

		match buf {
			None => Ok(()),
			Some(buf) => writer.write(buf),
		}?;

		if wants_flush {
			writer.flush()?;
		}

		Ok(())
	}
}

impl<'a, W: AsyncWrite + ?Sized> AsyncWrite
	for WriteTransaction<'a, W>
{
	type Error = W::Error;

	fn poll_write_slice(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, Self::Error>> {
		match self.buf.as_mut() {
			None => {
				Pin::new(&mut self.writer).poll_write_slice(cx, buf)
			},
			Some(x) => {
				x.extend_from_slice(buf);
				Poll::Ready(Ok(buf.len()))
			},
		}
	}

	fn poll_flush_once(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		match self.buf {
			None => Pin::new(&mut self.writer).poll_flush_once(cx),
			Some(_) => {
				self.wants_flush = true;
				Poll::Ready(Ok(()))
			},
		}
	}
}

impl<'a, W: AsyncWrite + ?Sized> WriteTransaction<'a, W> {
	/// Finish the transaction.
	///
	/// If the transaction does buffering then this method will attempt to write
	/// the whole buffer to the writer. Additionally, if [`flush()`] was ever
	/// called on the transaction, then this method will also attempt to flush
	/// the writer after writing to it. However, if the transaction is was created
	/// with [`unbuffered()`] then this method is simply a no-op. It is semantically
	/// correct that a [`WriteTransaction`] is always finished regardless of whether
	/// the finish operation does any work.
	///
	/// [`flush()`]: fn@AsyncWrite::flush
	/// [`unbuffered()`]: fn@WriteTransaction::unbuffered
	pub async fn finish_async(self) -> Result<(), W::Error> {
		let Self { buf, wants_flush, writer } = self;

		match buf {
			None => Ok(()),
			Some(buf) => writer.write(buf).await,
		}?;

		if wants_flush {
			writer.flush().await?;
		}

		Ok(())
	}
}
