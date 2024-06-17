use core::pin::Pin;
use core::task::{ready, Context, Poll};

use alloc::vec::Vec;

use crate::transaction::{AsyncWriteTransaction, WriteTransaction};
use crate::{AsyncWrite, AsyncWriteExt, Write, WriteExt};

/// A buffered write transaction.
///
/// Data written to this transaction will be stored in an internal `Vec<u8>` and
/// will be written to the underlying writer when it is finished. Flushing the
/// transaction will instruct it to also flush the underlying writer after it writes
/// the data to it.
#[derive(Debug)]
pub struct Buffered<'a, W> {
	writer: W,
	buf: &'a mut Vec<u8>,
	wants_flush: bool,
}

impl<'a, W> Buffered<'a, W> {
	/// Create a new [`Buffered`] transaction.
	///
	/// `buf` is where data will be buffered in before it is written out.
	///
	/// # Panics
	///
	/// If `buf` is not empty.
	pub fn new(writer: W, buf: &'a mut Vec<u8>) -> Self {
		assert!(buf.is_empty(), "buf should be empty");
		Self { writer, buf, wants_flush: false }
	}

	/// Get a reference to the underlying writer.
	pub fn writer(&self) -> &W {
		&self.writer
	}

	/// Get a mutable reference to the underlying writer.
	pub fn writer_mut(&mut self) -> &mut W {
		&mut self.writer
	}
}

impl<'a, W> Write for Buffered<'a, W>
where
	W: Write,
{
	type Error = W::Error;

	fn write_slice(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		self.buf.extend_from_slice(buf);
		Ok(buf.len())
	}

	fn flush_once(&mut self) -> Result<(), Self::Error> {
		self.wants_flush = true;
		Ok(())
	}
}

impl<'a, W> WriteTransaction for Buffered<'a, W>
where
	W: Write,
{
	fn finish(self) -> Result<(), Self::Error> {
		let Self { buf, wants_flush, mut writer } = self;

		writer.write(buf.as_slice())?;

		if wants_flush {
			writer.flush()?;
		}

		Ok(())
	}
}

impl<'a, W> AsyncWrite for Buffered<'a, W>
where
	W: AsyncWrite + Unpin,
{
	type Error = W::Error;

	fn poll_write_slice(
		mut self: Pin<&mut Self>,
		_: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, Self::Error>> {
		self.buf.extend_from_slice(buf);
		Poll::Ready(Ok(buf.len()))
	}

	fn poll_flush_once(
		mut self: Pin<&mut Self>,
		_: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		self.wants_flush = true;
		Poll::Ready(Ok(()))
	}
}

impl<'a, W> AsyncWriteTransaction for Buffered<'a, W>
where
	W: AsyncWrite + Unpin,
{
	fn poll_finish(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		let Self { ref mut buf, wants_flush, ref mut writer } = *self;
		let buf = &mut **buf;

		ready!(Pin::new(&mut *writer).poll_write(cx, buf.as_slice()))?;

		if wants_flush {
			ready!(Pin::new(&mut *writer).poll_flush(cx))?;
		}

		Poll::Ready(Ok(()))
	}
}
