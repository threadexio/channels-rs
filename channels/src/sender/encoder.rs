use alloc::vec::Vec;

use channels_packet::{Frame, FrameNumSequence, Payload};

use crate::error::EncodeError;

use super::config::Config;

/// Encoder for the channels protocol.
#[derive(Debug, Default)]
pub struct Encoder {
	config: Config,
	seq: FrameNumSequence,
}

impl Encoder {
	/// Create a new encoder with `config`.
	#[inline]
	#[must_use]
	pub const fn with_config(config: Config) -> Self {
		Self { config, seq: FrameNumSequence::new() }
	}

	/// Get the configuration of this encoder.
	#[inline]
	pub fn config(&self) -> &Config {
		&self.config
	}
}

impl crate::io::framed::Encoder for Encoder {
	type Item = Vec<u8>;
	type Error = EncodeError;

	fn encode(
		&mut self,
		item: Self::Item,
		buf: &mut Vec<u8>,
	) -> Result<(), Self::Error> {
		let payload =
			Payload::new(item).map_err(|_| EncodeError::TooLarge)?;

		let frame = Frame::builder()
			.frame_num_from_seq(&mut self.seq)
			.payload(payload);

		let header = frame.header().to_bytes();
		let payload = frame.payload;

		let frame_len = usize::checked_add(
			header.as_ref().len(),
			payload.as_slice().len(),
		)
		.ok_or(EncodeError::TooLarge)?;

		buf.reserve(frame_len);
		buf.extend_from_slice(header.as_ref());
		buf.extend_from_slice(payload.as_slice());
		Ok(())
	}
}
