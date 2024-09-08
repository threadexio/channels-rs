use core::fmt;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use alloc::vec::Vec;

use pin_project::pin_project;

use crate::buf::Cursor;
use crate::framed::Decoder;
use crate::source::{AsyncSource, Source};
use crate::util::slice_uninit_assume_init_mut;
use crate::util::Error;
use crate::{AsyncRead, AsyncReadExt, Read, ReadExt};

/// Convert a [`Read`] to a [`Source`] or an [`AsyncRead`] to an [`AsyncSource`].
///
/// This abstraction converts unstructured input streams to structured typed streams.
/// It reads raw bytes from a reader and processes them into structured data with the
/// help of a [`Decoder`]. The [`Decoder`] decides how the raw bytes are converted back
/// to items. At the other end of the stream, a [`FramedWrite`] will have produced the bytes
/// with a matching [`Encoder`].
///
/// The [`Decoder`] will read bytes from the reader via an intermediary buffer owned by the
/// [`FramedRead`] instance, the "read buffer". The [`Decoder`] will remove bytes from it
/// as each item is decoded.
///
/// [`FramedWrite`]: crate::framed::FramedWrite
/// [`Encoder`]: crate::framed::Encoder
#[pin_project]
#[derive(Debug)]
pub struct FramedRead<R, D> {
	#[pin]
	reader: R,
	decoder: D,
	buf: Vec<u8>,
}

impl<R, D> FramedRead<R, D> {
	/// Create a new [`FramedRead`].
	#[inline]
	#[must_use]
	pub const fn new(reader: R, decoder: D) -> Self {
		Self { reader, decoder, buf: Vec::new() }
	}

	/// Create a new [`FramedRead`] that can hold `capacity` bytes in its read buffer
	/// before allocating.
	#[inline]
	#[must_use]
	pub fn with_capacity(
		reader: R,
		decoder: D,
		capacity: usize,
	) -> Self {
		Self { reader, decoder, buf: Vec::with_capacity(capacity) }
	}

	/// Get a reference to the underlying reader.
	#[inline]
	#[must_use]
	pub fn reader(&self) -> &R {
		&self.reader
	}

	/// Get a mutable reference to the underlying reader.
	#[inline]
	#[must_use]
	pub fn reader_mut(&mut self) -> &mut R {
		&mut self.reader
	}

	/// Get a pinned reference to the underlying reader.
	#[inline]
	#[must_use]
	pub fn reader_pin_mut(self: Pin<&mut Self>) -> Pin<&mut R> {
		self.project().reader
	}

	/// Get a reference to the underlying decoder.
	#[inline]
	#[must_use]
	pub fn decoder(&self) -> &D {
		&self.decoder
	}

	/// Get a mutable reference to the decoder.
	#[inline]
	#[must_use]
	pub fn decoder_mut(&mut self) -> &mut D {
		&mut self.decoder
	}

	/// Get a reference to the decoder from a pinned reference of the [`FramedRead`].
	#[inline]
	#[must_use]
	pub fn decoder_pin_mut(self: Pin<&mut Self>) -> &mut D {
		self.project().decoder
	}

	/// Get a reference to the read buffer.
	#[inline]
	#[must_use]
	pub fn read_buffer(&self) -> &Vec<u8> {
		&self.buf
	}

	/// Get a mutable reference to the read buffer.
	#[inline]
	#[must_use]
	pub fn read_buffer_mut(&mut self) -> &mut Vec<u8> {
		&mut self.buf
	}

	/// Get a reference to the read buffer from a pinned reference of the [`FramedRead`].
	#[inline]
	#[must_use]
	pub fn map_decoder<T, F>(self, f: F) -> FramedRead<R, T>
	where
		T: Decoder,
		F: FnOnce(D) -> T,
	{
		FramedRead {
			reader: self.reader,
			decoder: f(self.decoder),
			buf: self.buf,
		}
	}

	/// Destruct this [`FramedRead`] and get back the reader, dropping the decoder.
	#[inline]
	#[must_use]
	pub fn into_reader(self) -> R {
		self.reader
	}

	/// Destruct this [`FramedRead`] and get back the decoder, dropping the reader.
	#[inline]
	#[must_use]
	pub fn into_decoder(self) -> D {
		self.decoder
	}

	/// Destruct this [`FramedRead`] and get back both the decoder and the reader.
	#[inline]
	#[must_use]
	pub fn into_inner(self) -> (R, D) {
		(self.reader, self.decoder)
	}
}

/// Errors when receiving an item over a [`FramedRead`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramedReadError<T, Io> {
	/// The decoder returned an error.
	Decode(T),
	/// There was an I/O error.
	Io(Io),
}

impl<T, Io> fmt::Display for FramedReadError<T, Io>
where
	T: fmt::Display,
	Io: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Decode(e) => e.fmt(f),
			Self::Io(e) => e.fmt(f),
		}
	}
}

impl<T, Io> Error for FramedReadError<T, Io> where
	Self: fmt::Debug + fmt::Display
{
}

impl<R, D> AsyncSource for FramedRead<R, D>
where
	R: AsyncRead,
	D: Decoder,
{
	type Item =
		Result<D::Output, FramedReadError<D::Error, R::Error>>;

	fn poll_next(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Self::Item> {
		let mut this = self.project();

		loop {
			match this.decoder.decode(this.buf) {
				Ok(Some(x)) => return Poll::Ready(Ok(x)),
				Ok(None) => {
					if this.buf.spare_capacity_mut().is_empty() {
						this.buf.reserve(1);
					}

					let n = {
						let mut buf =
							Cursor::new(get_vec_spare_cap(this.buf));

						ready!(this
							.reader
							.as_mut()
							.poll_read_buf(cx, &mut buf))
						.map_err(FramedReadError::Io)?;

						buf.pos()
					};

					unsafe {
						this.buf.set_len(this.buf.len() + n);
					}
				},
				Err(e) => {
					return Poll::Ready(Err(
						FramedReadError::Decode(e),
					));
				},
			}
		}
	}
}

impl<R, D> Source for FramedRead<R, D>
where
	R: Read,
	D: Decoder,
{
	type Item =
		Result<D::Output, FramedReadError<D::Error, R::Error>>;

	fn next(&mut self) -> Self::Item {
		loop {
			match self.decoder.decode(&mut self.buf) {
				Ok(Some(x)) => return Ok(x),
				Ok(None) => {
					if self.buf.spare_capacity_mut().is_empty() {
						self.buf.reserve(1);
					}

					let n = {
						let mut buf = Cursor::new(get_vec_spare_cap(
							&mut self.buf,
						));

						self.reader
							.read_buf(&mut buf)
							.map_err(FramedReadError::Io)?;

						buf.pos()
					};

					unsafe {
						self.buf.set_len(self.buf.len() + n);
					}
				},
				Err(e) => return Err(FramedReadError::Decode(e)),
			}
		}
	}
}

fn get_vec_spare_cap(vec: &mut Vec<u8>) -> &mut [u8] {
	unsafe { slice_uninit_assume_init_mut(vec.spare_capacity_mut()) }
}

#[cfg(test)]
mod tests {
	use super::*;

	use core::convert::Infallible;

	use crate::buf::{Buf, Cursor};
	use crate::source::Source;

	struct U32Decoder;

	impl Decoder for U32Decoder {
		type Output = i32;
		type Error = Infallible;

		fn decode(
			&mut self,
			buf: &mut Vec<u8>,
		) -> Result<Option<Self::Output>, Self::Error> {
			let x = match buf.get(..4) {
				Some(x) => x.try_into().expect(""),
				None => return Ok(None),
			};

			let x = i32::from_be_bytes(x);

			buf.drain(..4);
			Ok(Some(x))
		}
	}

	#[test]
	fn test_framed_read_eof() {
		let reader = Cursor::new([0u8, 0, 0, 42, 0, 0]).reader();
		let mut framed = FramedRead::new(reader, U32Decoder);

		assert_eq!(Source::next(&mut framed).expect(""), 42);
		assert!(matches!(
			Source::next(&mut framed),
			Err(FramedReadError::Io(_))
		));
	}

	#[test]
	fn test_framed_read_eof_exact() {
		let reader =
			Cursor::new([0u8, 0, 0, 42, 0, 0, 0, 62]).reader();
		let mut framed = FramedRead::new(reader, U32Decoder);

		assert_eq!(Source::next(&mut framed).expect(""), 42);
		assert_eq!(Source::next(&mut framed).expect(""), 62);
		assert!(matches!(
			Source::next(&mut framed),
			Err(FramedReadError::Io(_))
		));
	}
}
