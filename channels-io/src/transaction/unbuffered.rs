use core::pin::Pin;
use core::task::{Context, Poll};

use pin_project::pin_project;

use crate::transaction::{AsyncWriteTransaction, WriteTransaction};
use crate::{AsyncWrite, Write};

/// An unbuffered transaction that does no buffering of data.
///
/// [`Unbuffered`] transactions proxy IO calls directly to the underlying writer
/// in a "1-1" fashion.
#[derive(Debug)]
#[pin_project]
pub struct Unbuffered<W> {
	#[pin]
	writer: W,
}

impl<W> Unbuffered<W> {
	/// Create a new [`Unbuffered`] transaction.
	pub fn new(writer: W) -> Self {
		Self { writer }
	}

	/// Get a reference to the underlying writer.
	#[must_use]
	pub fn writer(&self) -> &W {
		&self.writer
	}

	/// Get a mutable reference to the underlying writer.
	#[must_use]
	pub fn writer_mut(&mut self) -> &mut W {
		&mut self.writer
	}

	/// Get a pinned mutable reference to the underlying writer.
	#[must_use]
	pub fn writer_pin_mut(self: Pin<&mut Self>) -> Pin<&mut W> {
		self.project().writer
	}
}

impl<W> Write for Unbuffered<W>
where
	W: Write,
{
	type Error = W::Error;

	fn write_slice(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		self.writer.write_slice(buf)
	}

	fn flush_once(&mut self) -> Result<(), Self::Error> {
		self.writer.flush_once()
	}
}

impl<W> WriteTransaction for Unbuffered<W>
where
	W: Write,
{
	fn finish(self) -> Result<(), Self::Error> {
		Ok(())
	}
}

impl<W> AsyncWrite for Unbuffered<W>
where
	W: AsyncWrite,
{
	type Error = W::Error;

	fn poll_write_slice(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, Self::Error>> {
		let this = self.project();
		this.writer.poll_write_slice(cx, buf)
	}

	fn poll_flush_once(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		let this = self.project();
		this.writer.poll_flush_once(cx)
	}
}

impl<W> AsyncWriteTransaction for Unbuffered<W>
where
	W: AsyncWrite,
{
	fn poll_finish(
		self: Pin<&mut Self>,
		_: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}
}
