//! [`Frame`] and helper types.

use core::marker::PhantomData;

use crate::flags::Flags;
use crate::header::Header;
use crate::payload::Payload;
use crate::seq::{FrameNum, FrameNumSequence};

/// A protocol frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame<T> {
	/// Frame flags.
	pub flags: Flags,
	/// Frame number.
	pub frame_num: FrameNum,
	/// Frame payload data.
	pub payload: Payload<T>,
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
			flags: self.flags,
			frame_num: self.frame_num,
			payload: self.payload.as_ref(),
		}
	}

	/// Convert a [`&mut Frame<T>`] to a [`Frame<&mut T>`].
	///
	/// [`&mut Frame<T>]: Frame
	/// [`Frame<&mut T>`]: Frame
	#[inline]
	pub fn as_mut(&mut self) -> Frame<&mut T> {
		Frame {
			flags: self.flags,
			frame_num: self.frame_num,
			payload: self.payload.as_mut(),
		}
	}
}

impl<T: AsRef<[u8]>> Frame<T> {
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
	pub fn header(&self) -> Header {
		Header {
			flags: self.flags,
			frame_num: self.frame_num,
			data_len: self.payload.length(),
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
	pub const fn payload(self, payload: Payload<T>) -> Frame<T> {
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
