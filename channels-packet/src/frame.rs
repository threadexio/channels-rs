//! [`Frame`] and helper types.

use core::fmt;
use core::marker::PhantomData;
use core::ops::Range;

use crate::flags::Flags;
use crate::header::{Header, HeaderError};
use crate::payload::Payload;
use crate::seq::{FrameNum, FrameNumSequence};
use crate::util::Error;
use crate::wants::Wants;

/// A protocol frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame<T> {
	/// Frame flags.
	pub flags: Flags,
	/// Frame number.
	pub frame_num: FrameNum,
	/// Frame payload data.
	pub payload: T,
}

impl<T> Frame<T> {
	/// Create a new [`Builder`].
	#[inline]
	pub const fn builder() -> Builder<T> {
		Builder::new()
	}

	/// Convert a `Frame<T>` to a `Frame<U>` via a function _f_.
	#[inline]
	pub fn map_payload<U, F>(self, f: F) -> Frame<U>
	where
		F: FnOnce(T) -> U,
	{
		Frame {
			flags: self.flags,
			frame_num: self.frame_num,
			payload: f(self.payload),
		}
	}

	fn get_header(&self, data_len: u32) -> Header {
		Header {
			flags: self.flags,
			frame_num: self.frame_num,
			data_len,
		}
	}
}

/// Errors when parsing a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameError {
	/// There was an error while parsing the header.
	Header(HeaderError),
	/// The frame is too large to fit in memory.
	TooLarge,
}

impl fmt::Display for FrameError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Header(e) => e.fmt(f),
			Self::TooLarge => f.write_str("frame too large"),
		}
	}
}

impl Error for FrameError {}

impl Frame<Range<usize>> {
	/// Try to parse a frame from `bytes` returning an indice to the payload.
	///
	/// Returns a [`Frame`] whose payload is a [`Range`] inside `bytes`. If `bytes` does not
	/// have a complete frame, this method returns `Ok(Err(Wants))`, the provided [`Wants`]
	/// type can be used as an optimization hint for outside code. If `bytes` has a complete
	/// frame but that frame contains errors or cannot be parsed, it returns `Err(...)`.
	/// Otherwise, it returns `Ok(Ok(frame))`.
	///
	/// The implementation makes no assumptions about the platform's pointer width and thus
	/// it is possible for parsing to fail if the entire frame is larger than the address
	/// space of the platform. Typically this is only a concern on 16-bit and 32-bit targets.
	pub fn try_parse_range(
		bytes: &[u8],
	) -> Result<Result<Self, Wants>, FrameError> {
		let header = match Header::try_parse(bytes) {
			Ok(Ok(x)) => x,
			Ok(Err(wants)) => return Ok(Err(wants)),
			Err(e) => return Err(FrameError::Header(e)),
		};

		let data_len = header
			.data_len
			.try_into()
			.map_err(|_| FrameError::TooLarge)?;

		let frame_len = Header::SIZE
			.checked_add(data_len)
			.ok_or(FrameError::TooLarge)?;

		if bytes.len() < frame_len {
			return Ok(Err(Wants(frame_len - bytes.len())));
		}

		Ok(Ok(Frame {
			flags: header.flags,
			frame_num: header.frame_num,
			payload: Header::SIZE..frame_len,
		}))
	}

	/// Get the length of the frame in bytes.
	///
	/// Returns `None` if the sum of the lengths of the header and payload cannot be
	/// represented in a `usize`. Otherwise, it returns `Some(length)`.
	#[inline]
	#[must_use]
	pub fn length(&self) -> Option<usize> {
		calculate_frame_len(self.payload.len())
	}

	/// Get the header of the frame.
	///
	/// Returns `None` if the range of the payload does not fit inside a [`u32`].
	///
	/// # Example
	///
	/// ```
	/// # use core::ops::Range;
	/// # use channels_packet::{Frame, Header, FrameNum, Flags};
	/// let frame = Frame {
	///     flags: Flags::empty(),
	///     frame_num: FrameNum::new(13),
	///     payload: 8..42,
	/// };
	///
	/// assert_eq!(frame.header().unwrap(), Header {
	///     flags: Flags::empty(),
	///     frame_num: FrameNum::new(13),
	///     data_len: 34,
	/// });
	/// ```
	#[inline]
	#[must_use]
	pub fn header(&self) -> Option<Header> {
		let len = self.payload.len().try_into().ok()?;
		Some(self.get_header(len))
	}
}

impl<'a> Frame<Payload<&'a [u8]>> {
	/// Try to parse a frame from `bytes` returning the payload as a slice.
	///
	/// Returns a [`Frame`] whose payload is a slice of `bytes`. If `bytes` does not
	/// have a complete frame, this method returns `Ok(Err(Wants))`, the provided [`Wants`]
	/// type can be used as an optimization hint for outside code. If `bytes` has a complete
	/// frame but that frame contains errors or cannot be parsed, it returns `Err(...)`.
	/// Otherwise, it returns `Ok(Ok(frame))`.
	///
	/// The implementation makes no assumptions about the platform's pointer width and thus
	/// it is possible for parsing to fail if the entire frame is larger than the address
	/// space of the platform. Typically this is only a concern on 16-bit and 32-bit targets.
	#[allow(clippy::missing_panics_doc)]
	pub fn try_parse(
		bytes: &'a [u8],
	) -> Result<Result<Self, Wants>, FrameError> {
		let frame = match Frame::try_parse_range(bytes) {
			Ok(Ok(x)) => x,
			Ok(Err(wants)) => return Ok(Err(wants)),
			Err(e) => return Err(e),
		};

		Ok(Ok(frame.map_payload(|x| {
			// SAFETY: `try_parse_range` returns a range to the payload of the frame, so
			//         so the length of the slice represented by that range is always a
			//         32 bit number. If this `u32` number cannot be represented using the
			//         platform's `usize`, then `try_parse_range` will fail. If `try_parse_range`
			//         succeeds, then we know that the length of the payload fits inside
			//         a `usize`. We also know that it is a valid `u32` number. `Payload::new`
			//         fails if the length of the payload cannot fit inside a `u32` or a
			//         `usize`, whichever is smaller.
			Payload::new(&bytes[x]).expect(
				"try_parse_range returned an invalid payload range",
			)
		})))
	}
}

impl<T: AsRef<[u8]>> Frame<Payload<T>> {
	/// Get the length of the frame in bytes.
	///
	/// Returns `None` if the sum of the lengths of the header and payload cannot be
	/// represented in a `usize`. Otherwise, it returns `Some(length)`.
	#[inline]
	#[must_use]
	pub fn length(&self) -> Option<usize> {
		calculate_frame_len(self.payload.get().as_ref().len())
	}

	/// Get the header of the frame.
	///
	/// # Example
	///
	/// ```
	/// # use channels_packet::{Frame, Payload, Header, FrameNum, Flags};
	/// let frame = Frame {
	///     flags: Flags::empty(),
	///     frame_num: FrameNum::new(13),
	///     payload: Payload::new([1, 2, 3, 4]).unwrap(),
	/// };
	///
	/// assert_eq!(frame.header(), Header {
	///     flags: Flags::empty(),
	///     frame_num: FrameNum::new(13),
	///     data_len: 4,
	/// });
	/// ```
	#[inline]
	#[must_use]
	pub fn header(&self) -> Header {
		self.get_header(self.payload.length())
	}
}

impl<T> Frame<Payload<T>> {
	/// Convert a `&Frame<Payload<T>>` to a `Frame<Payload<&T>>`.
	#[inline]
	#[must_use]
	pub fn as_ref(&self) -> Frame<Payload<&T>> {
		Frame {
			flags: self.flags,
			frame_num: self.frame_num,
			payload: self.payload.as_ref(),
		}
	}

	/// Convert a `&mut Frame<Payload<T>>` to a `Frame<Payload<&mut T>>`.
	///
	/// [`&mut Frame<T>]: Frame
	/// [`Frame<&mut T>`]: Frame
	#[inline]
	#[must_use]
	pub fn as_mut(&mut self) -> Frame<Payload<&mut T>> {
		Frame {
			flags: self.flags,
			frame_num: self.frame_num,
			payload: self.payload.as_mut(),
		}
	}
}

/// [`Frame`] builder.
///
/// # Example
///
/// ```no_run
/// # use channels_packet::{frame::{Builder, Frame}, Payload, FrameNum};
/// let payload = Payload::new([1u8, 1, 1, 1]).unwrap();
///
/// let mut frame = Builder::new()
///     .frame_num(FrameNum::new(0))
///     .payload(payload);
/// ```
#[allow(missing_debug_implementations)]
#[must_use = "builders don't do anything unless you build them"]
pub struct Builder<T> {
	_marker: PhantomData<T>,
	flags: Flags,
	frame_num: FrameNum,
}

impl<T> Builder<T> {
	/// Create a new [`Builder`].
	#[inline]
	pub const fn new() -> Self {
		Self {
			_marker: PhantomData,
			flags: Flags::empty(),
			frame_num: FrameNum::new(0),
		}
	}

	/// Set the frame flags.
	///
	/// # Example
	///
	/// ```no_run
	/// # use channels_packet::{frame::Builder, Payload, Flags};
	/// let frame = Builder::new()
	///     // ...
	///     .flags(Flags::empty())
	///     // ...
	/// #   .payload(Payload::new([]).unwrap());
	/// ```
	#[inline]
	pub const fn flags(mut self, flags: Flags) -> Self {
		self.flags = flags;
		self
	}

	/// Set the frame number.
	///
	/// # Example
	///
	/// ```no_run
	/// # use channels_packet::{frame::Builder, Payload, FrameNum};
	/// let frame = Builder::new()
	///     // ...
	///     .frame_num(FrameNum::new(23))
	///     // ...
	/// #   .payload(Payload::new([]).unwrap());
	/// ```
	#[inline]
	pub const fn frame_num(mut self, frame_num: FrameNum) -> Self {
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
	/// # use channels_packet::{frame::Builder, Payload, FrameNumSequence};
	/// let mut seq = FrameNumSequence::new();
	///
	/// let frame = Builder::new()
	///     // ...
	///     .frame_num_from_seq(&mut seq)
	///     // ...
	/// #   .payload(Payload::new([]).unwrap());
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
	/// # use channels_packet::{frame::Builder, Payload};
	/// let buf: [u8; 6] = [1, 2, 3, 4, 5, 6];
	///
	/// let frame = Builder::new()
	///     // ...
	///     .payload(Payload::new(buf).unwrap());
	///
	/// assert_eq!(frame.payload.as_slice(), &[1, 2, 3, 4, 5, 6]);
	/// ```
	#[inline]
	pub const fn payload(self, payload: T) -> Frame<T> {
		let Self { _marker, flags, frame_num } = self;

		Frame { flags, frame_num, payload }
	}
}

impl<T> Default for Builder<T> {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}

const fn calculate_frame_len(payload_len: usize) -> Option<usize> {
	Header::SIZE.checked_add(payload_len)
}
