use core::pin::Pin;
use core::task::{ready, Context, Poll};

use alloc::vec::Vec;

use pin_project::pin_project;

use crate::buf::Cursor;
use crate::transaction::{AsyncWriteTransaction, WriteTransaction};
use crate::{AsyncWrite, AsyncWriteExt, Write, WriteExt};

/// A buffered write transaction.
///
/// Data written to this transaction will be stored in an internal `Vec<u8>` and
/// will be written to the underlying writer when it is finished. Flushing the
/// transaction will instruct it to also flush the underlying writer after it writes
/// the data to it.
#[derive(Debug)]
#[pin_project]
pub struct Buffered<'a, W> {
	#[pin]
	writer: W,
	buf: Cursor<&'a mut Vec<u8>>,
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
		Self { writer, buf: Cursor::new(buf), wants_flush: false }
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
		self.buf.get_mut().extend_from_slice(buf);
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

		writer.write(buf)?;

		if wants_flush {
			writer.flush()?;
		}

		Ok(())
	}
}

impl<'a, W> AsyncWrite for Buffered<'a, W>
where
	W: AsyncWrite,
{
	type Error = W::Error;

	fn poll_write_slice(
		self: Pin<&mut Self>,
		_: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, Self::Error>> {
		let this = self.project();
		this.buf.get_mut().extend_from_slice(buf);
		Poll::Ready(Ok(buf.len()))
	}

	fn poll_flush_once(
		self: Pin<&mut Self>,
		_: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		let this = self.project();
		*this.wants_flush = true;
		Poll::Ready(Ok(()))
	}
}

impl<'a, W> AsyncWriteTransaction for Buffered<'a, W>
where
	W: AsyncWrite,
{
	fn poll_finish(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		let mut this = self.project();

		ready!(this.writer.as_mut().poll_write(cx, this.buf))?;

		if *this.wants_flush {
			ready!(this.writer.poll_flush(cx))?;
		}

		Poll::Ready(Ok(()))
	}
}
