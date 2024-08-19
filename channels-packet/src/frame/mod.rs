//! TODO: docs

use core::fmt;
use core::marker::PhantomData;

use crate::header::{FrameNumSequence, Header, HeaderError};
use crate::num::u6;
use crate::payload::Payload;

mod encoded;

pub use self::encoded::Encoded;

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

	/// Convert a [`&Frame<T>`] to a [`Frame<&T>`].
	///
	/// [`&Frame<T>`]: Frame
	/// [`Frame<&T>`]: Frame
	#[inline]
	pub fn as_ref(&self) -> Frame<&T> {
		Frame {
			payload: self.payload.as_ref(),
			frame_num: self.frame_num,
		}
	}

	/// Convert a [`&mut Frame<T>`] to a [`Frame<&mut T>`].
	///
	/// [`&mut Frame<T>]: Frame
	/// [`Frame<&mut T>`]: Frame
	#[inline]
	pub fn as_mut(&mut self) -> Frame<&mut T> {
		Frame {
			payload: self.payload.as_mut(),
			frame_num: self.frame_num,
		}
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

	/// Encode the frame and return it in a [`Buf`].
	pub fn encode(self) -> Encoded<T> {
		Encoded::new(self.header(), self.payload)
	}
}

/// TODO: docs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameError {
	/// TODO: docs
	Header(HeaderError),
	/// TODO: docs
	TooLarge,
}

impl From<HeaderError> for FrameError {
	fn from(err: HeaderError) -> Self {
		Self::Header(err)
	}
}

impl fmt::Display for FrameError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match *self {
			Self::Header(e) => e.fmt(f),
			Self::TooLarge => f.write_str("payload too large"),
		}
	}
}

#[cfg(feature = "std")]
impl std::error::Error for FrameError {}

impl<'a> Frame<&'a [u8]> {
	/// TODO: docs
	pub fn try_parse(
		bytes: &'a [u8],
	) -> Result<Option<Self>, FrameError> {
		let hdr = match Header::try_parse(bytes) {
			Ok(Some(x)) => x,
			Ok(None) => return Ok(None),
			Err(e) => return Err(FrameError::from(e)),
		};

		let payload_start = hdr.length();

		let payload_len: usize = hdr
			.data_len
			.get()
			.try_into()
			.map_err(|_| FrameError::TooLarge)?;

		let payload_end = payload_start
			.checked_add(payload_len)
			.ok_or(FrameError::TooLarge)?;

		let Some(payload) = bytes.get(payload_start..payload_end)
		else {
			return Ok(None);
		};

		Ok(Some(Self {
			payload: unsafe { Payload::new_unchecked(payload) },
			frame_num: hdr.frame_num,
		}))
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
	_marker: PhantomData<T>,
	frame_num: u6,
}

impl<T> Builder<T> {
	/// Create a new [`Builder`].
	#[inline]
	pub const fn new() -> Self {
		Self { _marker: PhantomData, frame_num: u6::new_truncate(0) }
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
		let Self { _marker, frame_num } = self;

		Frame { payload, frame_num }
	}
}

impl<T> Default for Builder<T> {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}
