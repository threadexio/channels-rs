use core::fmt;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use alloc::vec::Vec;

use pin_project::pin_project;

use crate::buf::Cursor;
use crate::framed::Encoder;
use crate::sink::{AsyncSink, Sink};
use crate::{AsyncWrite, AsyncWriteExt, Write, WriteExt};

/// Convert a [`Write`] to a [`Sink`] or an [`AsyncWrite`] to an [`AsyncSink`].
///
/// This abstraction converts unstructured byte streams to structured typed output streams.
/// It accepts "items" that are then processed by an [`Encoder`] and sent to the writer.
/// The [`Encoder`] decides how the provided items are converted to bytes. At the other end
/// of the stream, a [`FramedRead`] will read the output of the encoder and using a
/// matching [`Decoder`] will decode the byte stream back into the items given to this
/// [`FramedWrite`].
///
/// The [`Encoder`] will write the encoded items into a buffer owned by the [`FramedWrite`]
/// instance, the "write buffer". This buffer will then be written to the underlying writer
/// as needed.
///
/// [`FramedRead`]: crate::framed::FramedRead
/// [`Decoder`]: crate::framed::Decoder
#[pin_project]
#[derive(Debug)]
pub struct FramedWrite<W, E> {
	#[pin]
	writer: W,
	encoder: E,
	buf: Vec<u8>,
}

impl<W, E> FramedWrite<W, E> {
	/// Create a new [`FramedWrite`].
	#[inline]
	#[must_use]
	pub const fn new(writer: W, encoder: E) -> Self {
		Self { writer, encoder, buf: Vec::new() }
	}

	/// Create a new [`FramedWrite`] that can hold `capacity` bytes in its write buffer
	/// before allocating.
	#[inline]
	#[must_use]
	pub fn with_capacity(
		writer: W,
		encoder: E,
		capacity: usize,
	) -> Self {
		Self { writer, encoder, buf: Vec::with_capacity(capacity) }
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

	/// Get a pinned reference to the underlying writer.
	#[inline]
	#[must_use]
	pub fn writer_pin_mut(self: Pin<&mut Self>) -> Pin<&mut W> {
		self.project().writer
	}

	/// Get a reference to the encoder.
	#[inline]
	#[must_use]
	pub fn encoder(&self) -> &E {
		&self.encoder
	}

	/// Get a mutable reference to the encoder.
	#[inline]
	#[must_use]
	pub fn encoder_mut(&mut self) -> &mut E {
		&mut self.encoder
	}

	/// Get a reference to the encoder from a pinned reference of the [`FramedWrite`].
	#[inline]
	#[must_use]
	pub fn encoder_pin_mut(self: Pin<&mut Self>) -> &mut E {
		self.project().encoder
	}

	/// Get a reference to the write buffer.
	#[inline]
	#[must_use]
	pub fn write_buffer(&self) -> &Vec<u8> {
		&self.buf
	}

	/// Get a mutable reference to the write buffer.
	#[inline]
	#[must_use]
	pub fn write_buffer_mut(&mut self) -> &mut Vec<u8> {
		&mut self.buf
	}

	/// Get a reference to the write buffer from a pinned reference of the [`FramedWrite`].
	#[inline]
	#[must_use]
	pub fn write_buffer_pin_mut(
		self: Pin<&mut Self>,
	) -> &mut Vec<u8> {
		self.project().buf
	}

	/// Consume the [`FramedWrite`] instance and return a new one that uses the same underlying
	/// writer and write buffer but with a new encoder.
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

	/// Destruct this [`FramedWrite`] and get back the writer, dropping the encoder.
	#[inline]
	#[must_use]
	pub fn into_writer(self) -> W {
		self.writer
	}

	/// Destruct this [`FramedWrite`] and get back the encoder, dropping the writer.
	#[inline]
	#[must_use]
	pub fn into_encoder(self) -> E {
		self.encoder
	}

	/// Destruct this [`FramedWrite`] and get back both the encoder and the writer.
	#[inline]
	#[must_use]
	pub fn into_inner(self) -> (W, E) {
		(self.writer, self.encoder)
	}
}

/// Errors when sending an item over a [`FramedWrite`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramedWriteError<T, Io> {
	/// The encoder returned an error.
	Encode(T),
	/// There was an I/O error.
	Io(Io),
}

impl<T, Io> fmt::Display for FramedWriteError<T, Io>
where
	T: fmt::Debug,
	Io: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Encode(e) => e.fmt(f),
			Self::Io(e) => e.fmt(f),
		}
	}
}

#[cfg(feature = "std")]
impl<T, Io> std::error::Error for FramedWriteError<T, Io> where
	Self: fmt::Debug + fmt::Display
{
}

impl<W, E> AsyncSink for FramedWrite<W, E>
where
	W: AsyncWrite + Unpin,
	E: Encoder + Unpin,
{
	type Item = E::Item;
	type Error = FramedWriteError<E::Error, W::Error>;

	fn poll_ready(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		let _ = cx;
		Poll::Ready(Ok(()))
	}

	fn start_send(
		self: Pin<&mut Self>,
		item: Self::Item,
	) -> Result<(), Self::Error> {
		let this = self.get_mut();
		this.encoder
			.encode(item, &mut this.buf)
			.map_err(FramedWriteError::Encode)
	}

	fn poll_flush(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		let mut this = self.project();

		while !this.buf.is_empty() {
			let mut buf = Cursor::new(this.buf.as_slice());
			let r =
				this.writer.as_mut().poll_write_buf_all(cx, &mut buf);

			let n = buf.pos();
			if n != 0 {
				this.buf.drain(..n);
			}

			ready!(r).map_err(FramedWriteError::Io)?;
		}

		ready!(this.writer.as_mut().poll_flush(cx))
			.map_err(FramedWriteError::Io)?;
		Poll::Ready(Ok(()))
	}
}

impl<W, E> Sink for FramedWrite<W, E>
where
	W: Write,
	E: Encoder,
{
	type Item = E::Item;

	type Error = FramedWriteError<E::Error, W::Error>;

	fn ready(&mut self) -> Result<(), Self::Error> {
		Ok(())
	}

	fn feed(&mut self, item: Self::Item) -> Result<(), Self::Error> {
		self.encoder
			.encode(item, &mut self.buf)
			.map_err(FramedWriteError::Encode)
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
		while !self.buf.is_empty() {
			let mut buf = Cursor::new(self.buf.as_slice());
			let r = self.writer.write_buf_all(&mut buf);

			let n = buf.pos();
			if n != 0 {
				self.buf.drain(..n);
			}

			r.map_err(FramedWriteError::Io)?;
		}

		self.writer.flush().map_err(FramedWriteError::Io)?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use core::convert::Infallible;

	use crate::buf::BufMut;
	use crate::sink::SinkExt;

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

		framed.send(42).expect("");
		framed.send(0xff_12_34).expect("");

		assert_eq!(buf.get(), &[0, 0, 0, 42, 0, 0xff, 0x12, 0x34, 0]);
	}

	#[test]
	fn test_framed_write_write_zero() {
		let mut buf = Cursor::new([0u8; 9]);
		let mut framed =
			FramedWrite::new(buf.by_ref().writer(), U32Encoder);

		framed.send(42).expect("");
		framed.send(0xff_12_34).expect("");
		assert!(matches!(
			framed.send(1),
			Err(FramedWriteError::Io(_))
		));
	}
}
