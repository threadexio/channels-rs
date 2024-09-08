use alloc::vec::Vec;

use channels_packet::frame::{Frame, FrameError};
use channels_packet::header::HeaderError;
use channels_packet::FrameNumSequence;

use crate::error::DecodeError;

use super::config::Config;

/// Decoder for the channels protocol.
#[derive(Debug, Default)]
pub struct Decoder {
	config: Config,
	seq: FrameNumSequence,
}

impl Decoder {
	/// Create a new decoder with `config`.
	#[inline]
	#[must_use]
	pub const fn with_config(config: Config) -> Self {
		Self { config, seq: FrameNumSequence::new() }
	}

	/// Get the configuration of this decoder.
	#[inline]
	pub fn config(&self) -> &Config {
		&self.config
	}
}

impl crate::io::framed::Decoder for Decoder {
	type Output = Vec<u8>;
	type Error = DecodeError;

	fn decode(
		&mut self,
		buf: &mut Vec<u8>,
	) -> Result<Option<Self::Output>, Self::Error> {
		let frame = match Frame::try_parse_range(buf) {
			Ok(Ok(x)) => x,
			Ok(Err(wants)) => {
				buf.reserve(wants.get());
				return Ok(None);
			},
			Err(e) => return Err(frame_to_decode_error(e)),
		};

		let payload_len = frame.payload.len();

		// SAFETY: `frame` was just parsed from `buf`. This means that the entirety of `frame`
		//         is contained in `buf`. And thus, the length of `frame` must fit inside
		//         a `usize`.
		let frame_len = frame
			.length()
			.expect("parsed frame should fit inside main memory");

		if let Some(max_size) = self.config.max_size {
			if payload_len > max_size.get() {
				return Err(DecodeError::TooLarge);
			}
		}

		if self.config.verify_order()
			&& frame.frame_num != self.seq.peek()
		{
			return Err(DecodeError::OutOfOrder);
		}

		let payload = buf[frame.payload.clone()].to_vec();

		let _ = self.seq.advance();
		buf.drain(..frame_len);
		Ok(Some(payload))
	}
}

#[inline]
const fn header_to_decode_error(err: HeaderError) -> DecodeError {
	use DecodeError as B;
	use HeaderError as A;

	match err {
		A::InvalidChecksum => B::InvalidChecksum,
		A::VersionMismatch => B::VersionMismatch,
	}
}

#[inline]
const fn frame_to_decode_error(err: FrameError) -> DecodeError {
	use DecodeError as B;
	use FrameError as A;

	match err {
		A::Header(e) => header_to_decode_error(e),
		A::TooLarge => B::TooLarge,
	}
}
