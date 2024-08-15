use core::pin::Pin;
use core::task::{ready, Context, Poll};

use pin_project::pin_project;

use crate::buf::Cursor;
use crate::error::WriteError;
use crate::framed::Encoder;
use crate::traits::{AsyncSink, Sink};
use crate::util::PollExt;
use crate::{AsyncWrite, AsyncWriteExt, Write, WriteExt};

/// TODO: docs
#[pin_project]
#[derive(Debug)]
pub struct FramedWrite<W, E> {
	#[pin]
	writer: W,
	encoder: E,
	buf: Vec<u8>,
}

impl<W, E> FramedWrite<W, E> {
	/// TODO: docs
	#[inline]
	#[must_use]
	pub const fn new(writer: W, encoder: E) -> Self {
		Self { writer, encoder, buf: Vec::new() }
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn with_capacity(
		writer: W,
		encoder: E,
		capacity: usize,
	) -> Self {
		Self { writer, encoder, buf: Vec::with_capacity(capacity) }
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn writer(&self) -> &W {
		&self.writer
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn writer_mut(&mut self) -> &mut W {
		&mut self.writer
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn writer_pin_mut(self: Pin<&mut Self>) -> Pin<&mut W> {
		self.project().writer
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn encoder(&self) -> &E {
		&self.encoder
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn encoder_mut(&mut self) -> &mut E {
		&mut self.encoder
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn encoder_pin_mut(self: Pin<&mut Self>) -> &mut E {
		self.project().encoder
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn map_encoder<T, F>(self, f: F) -> FramedWrite<W, T>
	where
		T: Encoder,
		F: FnOnce(E) -> T,
	{
		FramedWrite {
			writer: self.writer,
			encoder: f(self.encoder),
			buf: self.buf,
		}
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn write_buffer(&self) -> &Vec<u8> {
		&self.buf
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn write_buffer_mut(&mut self) -> &mut Vec<u8> {
		&mut self.buf
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn into_writer(self) -> W {
		self.writer
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn into_encoder(self) -> E {
		self.encoder
	}

	/// TODO: docs

	#[inline]
	#[must_use]
	pub fn into_inner(self) -> (W, E) {
		(self.writer, self.encoder)
	}
}

impl<W, E> FramedWrite<W, E>
where
	E: Encoder,
{
	fn encode_item(&mut self, item: E::Item) -> Result<(), E::Error> {
		self.encoder.encode(item, &mut self.buf)
	}

	fn poll_send_internal<F, Io>(
		self: Pin<&mut Self>,
		mut write_buf_all: F,
	) -> Poll<Result<(), Io>>
	where
		Io: WriteError,
		F: FnMut(
			Pin<&mut W>,
			&mut Cursor<&[u8]>,
		) -> Poll<Result<(), Io>>,
	{
		let mut this = self.project();

		while !this.buf.is_empty() {
			let mut buf = Cursor::new(this.buf.as_slice());
			let r = write_buf_all(this.writer.as_mut(), &mut buf);
			let n = buf.pos();

			if n != 0 {
				this.buf.drain(..n);
			}

			ready!(r)?;
		}

		Poll::Ready(Ok(()))
	}
}

/// TODO: docs
// TODO: error impl
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramedWriteError<T, Io> {
	/// TODO: docs
	Encode(T),
	/// TODO: docs
	Io(Io),
}

impl<W, E> AsyncSink for FramedWrite<W, E>
where
	W: AsyncWrite + Unpin,
	E: Encoder + Unpin,
{
	type Item = E::Item;
	type Error = FramedWriteError<E::Error, W::Error>;

	fn start_send(
		mut self: Pin<&mut Self>,
		item: Self::Item,
	) -> Result<(), Self::Error> {
		self.encode_item(item).map_err(FramedWriteError::Encode)
	}

	fn poll_send(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		self.poll_send_internal(|w, buf| {
			w.poll_write_buf_all(cx, buf)
		})
		.map_err(FramedWriteError::Io)
	}
}

impl<W, E> Sink for FramedWrite<W, E>
where
	W: Write,
	E: Encoder,
{
	type Item = E::Item;

	type Error = FramedWriteError<E::Error, W::Error>;

	fn send(&mut self, item: Self::Item) -> Result<(), Self::Error> {
		self.encode_item(item).map_err(FramedWriteError::Encode)?;

		let pinned = unsafe { Pin::new_unchecked(self) };

		pinned
			.poll_send_internal(|w, buf| unsafe {
				Poll::Ready(w.get_unchecked_mut().write_buf_all(buf))
			})
			.unwrap()
			.map_err(FramedWriteError::Io)?;

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::BufMut;

	use super::*;

	use core::convert::Infallible;

	struct U32Encoder;

	impl Encoder for U32Encoder {
		type Item = u32;
		type Error = Infallible;

		fn encode(
			&mut self,
			item: Self::Item,
			buf: &mut Vec<u8>,
		) -> Result<(), Self::Error> {
			buf.extend(&item.to_be_bytes());
			Ok(())
		}
	}

	#[test]
	fn test_framed_write() {
		let mut buf = Cursor::new([0u8; 9]);
		let mut framed =
			FramedWrite::new(buf.by_ref().writer(), U32Encoder);

		Sink::send(&mut framed, 42).expect("");
		Sink::send(&mut framed, 0xff_12_34).expect("");

		assert_eq!(buf.get(), &[0, 0, 0, 42, 0, 0xff, 0x12, 0x34, 0]);
	}

	#[test]
	#[should_panic]
	fn test_framed_write_write_zero() {
		let mut buf = Cursor::new([0u8; 9]);
		let mut framed =
			FramedWrite::new(buf.by_ref().writer(), U32Encoder);

		Sink::send(&mut framed, 42).expect("");
		Sink::send(&mut framed, 0xff_12_34).expect("");
		Sink::send(&mut framed, 1).expect("");
	}
}
