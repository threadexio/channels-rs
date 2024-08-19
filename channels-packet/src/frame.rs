//! TODO: docs

use core::fmt;
use core::marker::PhantomData;

use channels_io::buf::Buf;

use crate::header::{FrameNumSequence, Header, HeaderBytes};
use crate::num::u6;
use crate::payload::Payload;

/// A protocol frame.
#[derive(Clone, PartialEq, Eq)]
pub struct Frame<T> {
	/// Frame payload data.
	pub payload: Payload<T>,
	/// Frame number.
	pub frame_num: u6,
}

impl<T> Frame<T> {
	/// Create a new [`Builder`].
	#[inline]
	pub const fn builder() -> Builder<T> {
		Builder::new()
	}
}

impl<T: AsRef<[u8]>> Frame<T> {
	/// Get the header of the frame.
	///
	/// # Example
	///
	/// ```
	/// # use channels_packet::{Frame, Header, num::{u6, u48}};
	/// let frame = Frame {
	///     payload: [1u8, 2, 3, 4],
	///     frame_num: u6::new_truncate(13)
	/// };
	///
	/// assert_eq!(frame.header(), Header {
	///     data_len: u48::new_truncate(4),
	///     frame_num: u6::new_truncate(13),
	/// });
	/// ```
	#[inline]
	pub fn header(&self) -> Header {
		Header {
			data_len: self.payload.length(),
			frame_num: self.frame_num,
		}
	}

	/// Encode this frame by borrowing the payload.
	pub fn encode_ref(&self) -> Encoded<&T> {
		Encoded::new(self.header(), self.payload.as_ref())
	}

	/// Consume this frame and return its encoded representation.
	pub fn encode(self) -> Encoded<T> {
		Encoded::new(self.header(), self.payload)
	}
}

impl<T: AsRef<[u8]>> fmt::Debug for Frame<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Frame")
			.field("payload", &self.payload)
			.field("frame_num", &self.frame_num)
			.finish()
	}
}

/// [`Frame`] builder.
///
/// # Example
///
/// ```no_run
/// # use channels_packet::{frame::{Builder, Frame}, num::u6};
/// let mut frame = Builder::new()
///     .frame_num(u6::new_truncate(0))
///     .payload([1, 1, 1, 1]);
/// ```
#[allow(missing_debug_implementations)]
#[must_use = "builders don't do anything unless you build them"]
pub struct Builder<T> {
	payload: PhantomData<T>,
	frame_num: u6,
}

impl<T> Builder<T> {
	/// Create a new [`Builder`].
	#[inline]
	pub const fn new() -> Self {
		Self { payload: PhantomData, frame_num: u6::new_truncate(0) }
	}

	/// Set the frame number.
	///
	/// # Example
	///
	/// ```no_run
	/// # use channels_packet::{frame::Builder, num::u6};
	/// let frame = Builder::new()
	///     // ...
	///     .frame_num(u6::new_truncate(23))
	///     // ...
	/// #   .payload(());
	/// ```
	#[inline]
	pub const fn frame_num(mut self, frame_num: u6) -> Self {
		self.frame_num = frame_num;
		self
	}

	/// Set the frame number from the next one in `seq`.
	///
	/// This method will advance `seq`.
	///
	/// # Example
	///
	/// ```no_run
	/// # use channels_packet::{frame::Builder, header::FrameNumSequence};
	/// let mut seq = FrameNumSequence::new();
	///
	/// let frame = Builder::new()
	///     // ...
	///     .frame_num_from_seq(&mut seq)
	///     // ...
	/// #   .payload(());
	/// ```
	#[inline]
	pub fn frame_num_from_seq(
		self,
		seq: &mut FrameNumSequence,
	) -> Self {
		self.frame_num(seq.advance())
	}

	/// Set the payload of the frame
	///
	/// # Example
	///
	/// ```
	/// # use channels_packet::frame::Builder;
	/// let buf: [u8; 6] = [1, 2, 3, 4, 5, 6];
	///
	/// let frame = Builder::new()
	///     // ...
	///     .payload(buf);
	///
	/// assert_eq!(frame.payload, [1, 2, 3, 4, 5, 6]);
	/// ```
	#[inline]
	pub const fn payload(self, payload: Payload<T>) -> Frame<T> {
		Frame { payload, frame_num: self.frame_num }
	}
}

impl<T> Default for Builder<T> {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}

/// An encoded [`Frame`].
///
/// This struct is a [`Buf`] that contains the encoded frame.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Encoded<T: AsRef<[u8]>> {
	header: HeaderBytes,
	payload: Payload<T>,
	pos: usize,
}

impl<T: AsRef<[u8]>> Encoded<T> {
	fn new(header: Header, payload: Payload<T>) -> Self {
		Self { header: header.to_bytes(), payload, pos: 0 }
	}

	/// Get the length of the entire frame.
	#[inline]
	// `is_empty` doesn't really make sense within the context of `Encoded`. An
	// encoded frame is never 0 bytes in length.
	#[allow(clippy::len_without_is_empty)]
	pub fn len(&self) -> usize {
		self.header.len() + self.payload.as_slice().len()
	}
}

impl<T: AsRef<[u8]>> Buf for Encoded<T> {
	fn remaining(&self) -> usize {
		self.len() - self.pos
	}

	fn chunk(&self) -> &[u8] {
		let hdr = self.header.as_ref();
		let payload = self.payload.as_slice();

		if self.pos < hdr.len() {
			&hdr[self.pos..]
		} else {
			let pos = self.pos - hdr.len();
			&payload[pos..]
		}
	}

	fn advance(&mut self, n: usize) {
		assert!(n <= self.remaining(), "n must not be greater than the amount of remaining bytes");
		self.pos += n;
	}
}

impl<T: AsRef<[u8]>> fmt::Debug for Encoded<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Encoded")
			.field("header", &self.header)
			.field("payload", &self.payload)
			.field("pos", &self.pos)
			.finish()
	}
}
