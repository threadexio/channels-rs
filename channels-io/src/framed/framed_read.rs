use core::pin::{pin, Pin};
use core::task::{ready, Context, Poll};

use alloc::vec::Vec;

use pin_project::pin_project;

use crate::buf::Cursor;
use crate::error::ReadError;
use crate::framed::Decoder;
use crate::util::{slice_uninit_assume_init_mut, PollExt};
use crate::{AsyncRead, AsyncReadExt, Read, ReadExt};

/// TODO: docs
#[pin_project]
#[derive(Debug)]
pub struct FramedRead<R, D> {
	#[pin]
	reader: R,
	decoder: D,
	buf: Vec<u8>,
}

impl<R, D> FramedRead<R, D> {
	/// TODO: docs
	#[inline]
	#[must_use]
	pub const fn new(reader: R, decoder: D) -> Self {
		Self { reader, decoder, buf: Vec::new() }
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn with_capacity(
		reader: R,
		decoder: D,
		capacity: usize,
	) -> Self {
		Self { reader, decoder, buf: Vec::with_capacity(capacity) }
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn reader(&self) -> &R {
		&self.reader
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn reader_mut(&mut self) -> &mut R {
		&mut self.reader
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn reader_pin_mut(self: Pin<&mut Self>) -> Pin<&mut R> {
		self.project().reader
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn decoder(&self) -> &D {
		&self.decoder
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn decoder_mut(&mut self) -> &mut D {
		&mut self.decoder
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn decoder_pin_mut(self: Pin<&mut Self>) -> &mut D {
		self.project().decoder
	}

	/// TODO: docs
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

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn read_buffer(&self) -> &Vec<u8> {
		&self.buf
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn read_buffer_mut(&mut self) -> &mut Vec<u8> {
		&mut self.buf
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn into_reader(self) -> R {
		self.reader
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn into_decoder(self) -> D {
		self.decoder
	}

	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn into_inner(self) -> (R, D) {
		(self.reader, self.decoder)
	}
}

/// TODO: docs
// TODO: error impl
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramedReadError<T, Io> {
	/// TODO: docs
	Decode(T),
	/// TODO: docs
	Io(Io),
}

impl<R, D> FramedRead<R, D>
where
	D: Decoder,
{
	#[allow(clippy::type_complexity)]
	fn poll_next_internal<F, E>(
		self: Pin<&mut Self>,
		mut read_buf: F,
	) -> Poll<Result<D::Output, FramedReadError<D::Error, E>>>
	where
		E: ReadError,
		F: FnMut(
			Pin<&mut R>,
			&mut Cursor<&mut [u8]>,
		) -> Poll<Result<(), E>>,
	{
		let mut this = self.project();

		loop {
			match this.decoder.decode(this.buf) {
				Some(Ok(x)) => return Poll::Ready(Ok(x)),
				Some(Err(e)) => {
					return Poll::Ready(Err(FramedReadError::Decode(
						e,
					)))
				},
				None => {
					if this.buf.spare_capacity_mut().is_empty() {
						this.buf.reserve(1024);
					}

					ready!(poll_read_vec(
						this.reader.as_mut(),
						this.buf,
						&mut read_buf
					))
					.map_err(FramedReadError::Io)?;
				},
			}
		}
	}
}

impl<R, D> FramedRead<R, D>
where
	R: AsyncRead,
	D: Decoder,
{
	/// TODO: docs
	#[allow(clippy::type_complexity)]
	pub fn poll_next(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<D::Output, FramedReadError<D::Error, R::Error>>>
	{
		self.poll_next_internal(|r, buf| r.poll_read_buf(cx, buf))
	}
}

impl<R, D> FramedRead<R, D>
where
	R: Read,
	D: Decoder,
{
	/// TODO: docs
	pub fn next_frame(
		&mut self,
	) -> Result<D::Output, FramedReadError<D::Error, R::Error>> {
		let pinned = unsafe { Pin::new_unchecked(self) };

		pinned
			.poll_next_internal(|r, buf| unsafe {
				Poll::Ready(r.get_unchecked_mut().read_buf(buf))
			})
			.unwrap()
	}
}

fn poll_read_vec<R, E, F>(
	r: Pin<&mut R>,
	buf: &mut Vec<u8>,
	mut read_buf: F,
) -> Poll<Result<usize, E>>
where
	E: ReadError,
	F: FnMut(
		Pin<&mut R>,
		&mut Cursor<&mut [u8]>,
	) -> Poll<Result<(), E>>,
{
	unsafe {
		let spare_cap =
			slice_uninit_assume_init_mut(buf.spare_capacity_mut());
		if spare_cap.is_empty() {
			return Poll::Ready(Ok(0));
		}

		let n = {
			let mut buf = Cursor::new(spare_cap);
			ready!(read_buf(r, &mut buf))?;
			buf.pos()
		};

		let new_len = buf.len() + n;
		buf.set_len(new_len);

		Poll::Ready(Ok(n))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use core::convert::Infallible;

	use crate::{Buf, Cursor};

	struct U32Decoder;

	impl Decoder for U32Decoder {
		type Output = i32;
		type Error = Infallible;

		fn decode(
			&mut self,
			buf: &mut Vec<u8>,
		) -> Option<Result<Self::Output, Self::Error>> {
			let x = buf.get(..4)?.try_into().expect("");
			let x = i32::from_be_bytes(x);

			buf.drain(..4);
			Some(Ok(x))
		}
	}

	#[test]
	fn test_framed_read_eof() {
		let reader = Cursor::new([0u8, 0, 0, 42, 0, 0]).reader();
		let mut framed = FramedRead::new(reader, U32Decoder);

		assert_eq!(framed.next_frame().expect(""), 42);
		assert!(matches!(
			framed.next_frame(),
			Err(FramedReadError::Io(_))
		));
	}

	#[test]
	fn test_framed_read_eof_exact() {
		let reader =
			Cursor::new([0u8, 0, 0, 42, 0, 0, 0, 62]).reader();
		let mut framed = FramedRead::new(reader, U32Decoder);

		assert_eq!(framed.next_frame().expect(""), 42);
		assert_eq!(framed.next_frame().expect(""), 62);
		assert!(matches!(
			framed.next_frame(),
			Err(FramedReadError::Io(_))
		));
	}
}
