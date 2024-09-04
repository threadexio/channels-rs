use alloc::vec::Vec;

use channels_packet::header::{Header, HeaderError};
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
		let Some(hdr) = Header::try_parse(buf.as_slice())
			.map_err(header_to_decode_error)?
		else {
			buf.reserve(Header::SIZE - buf.len());
			return Ok(None);
		};

		let payload_len: usize = hdr
			.data_len
			.try_into()
			.map_err(|_| DecodeError::TooLarge)?;

		let frame_len = Header::SIZE
			.checked_add(payload_len)
			.ok_or(DecodeError::TooLarge)?;

		if let Some(max_size) = self.config.max_size {
			if payload_len > max_size.get() {
				return Err(DecodeError::TooLarge);
			}
		}

		if self.config.verify_order()
			&& hdr.frame_num != self.seq.peek()
		{
			return Err(DecodeError::OutOfOrder);
		}

		if buf.len() < frame_len {
			buf.reserve(frame_len - buf.len());
			return Ok(None);
		}

		let payload = buf[Header::SIZE..frame_len].to_vec();

		let _ = self.seq.advance();
		buf.drain(..frame_len);
		Ok(Some(payload))
	}
}

const fn header_to_decode_error(err: HeaderError) -> DecodeError {
	use DecodeError as B;
	use HeaderError as A;

	match err {
		A::InvalidChecksum => B::InvalidChecksum,
		A::VersionMismatch => B::VersionMismatch,
	}
}
