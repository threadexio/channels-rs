//! TODO: docs

use core::fmt;

use alloc::vec::Vec;

use channels_io::framed::{Decoder, Encoder};

use crate::frame::Frame;
use crate::header::{FrameNumSequence, Header, HeaderError};

/// TODO: docs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncodeError {
	/// TODO: docs
	TooLarge,
}

impl fmt::Display for EncodeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::TooLarge => f.write_str("too large"),
		}
	}
}

#[cfg(feature = "std")]
impl std::error::Error for EncodeError {}

/// TODO: docs
#[derive(Debug)]
pub struct FrameEncoder {
	seq: FrameNumSequence,
}

impl FrameEncoder {
	/// TODO: docs
	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		Self { seq: FrameNumSequence::new() }
	}
}

impl Encoder for FrameEncoder {
	type Item = Vec<u8>;
	type Error = EncodeError;

	fn encode(
		&mut self,
		item: Self::Item,
		buf: &mut Vec<u8>,
	) -> Result<(), Self::Error> {
		let frame =
			Frame { payload: item, frame_num: self.seq.peek() };

		let hdr = frame.header().ok_or(EncodeError::TooLarge)?;
		let len = frame.length().ok_or(EncodeError::TooLarge)?;

		buf.reserve(len);
		buf.extend(hdr.to_bytes().as_ref());
		buf.extend(frame.payload.as_slice());

		let _ = self.seq.advance();
		Ok(())
	}
}

/// TODO: docs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecodeError {
	/// TODO: docs
	VersionMismatch,
	/// TODO: docs
	InvalidChecksum,
	/// TODO: docs
	OutOfOrder,
	/// TODO: docs
	TooLarge,
}

impl fmt::Display for DecodeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidChecksum => f.write_str("invalid checksum"),
			Self::OutOfOrder => f.write_str("out of order"),
			Self::TooLarge => f.write_str("too large"),
			Self::VersionMismatch => f.write_str("version mismatch"),
		}
	}
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}

/// TODO: docs
#[derive(Debug)]
pub struct FrameDecoder {
	seq: FrameNumSequence,
}

impl FrameDecoder {
	/// TODO: docs
	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		Self { seq: FrameNumSequence::new() }
	}
}

impl Decoder for FrameDecoder {
	type Output = Vec<u8>;
	type Error = DecodeError;

	fn decode(
		&mut self,
		buf: &mut Vec<u8>,
	) -> Option<Result<Self::Output, Self::Error>> {
		if buf.len() < 4 {
			buf.reserve(4 - buf.len());
			return None;
		}

		let hdr = match Header::try_from(buf.as_slice()) {
			Ok(x) => x,
			Err(HeaderError::NotEnough) => {
				buf.reserve(10 - buf.len());
				return None;
			},
			Err(HeaderError::InvalidChecksum) => {
				return Some(Err(DecodeError::InvalidChecksum))
			},
			Err(HeaderError::VersionMismatch) => {
				return Some(Err(DecodeError::VersionMismatch))
			},
		};

		if hdr.frame_num != self.seq.peek() {
			return Some(Err(DecodeError::OutOfOrder));
		}

		let hdr_len = hdr.length();

		let payload_len: usize = match hdr.data_len.get().try_into() {
			Ok(x) => x,
			Err(_) => return Some(Err(DecodeError::TooLarge)),
		};

		let Some(frame_len) = hdr_len.checked_add(payload_len) else {
			return Some(Err(DecodeError::TooLarge));
		};

		if buf.len() < frame_len {
			buf.reserve(frame_len - buf.len());
			return None;
		}

		let payload = buf[hdr_len..frame_len].to_vec();
		buf.drain(..frame_len);
		let _ = self.seq.advance();

		Some(Ok(payload))
	}
}
