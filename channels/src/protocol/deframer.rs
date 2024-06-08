use core::mem;

use alloc::vec::Vec;

use channels_packet::header::{
	Header, VerifyError, VerifyId, WithChecksum,
};
use channels_packet::id::IdSequence;
use channels_packet::Flags;

use crate::receiver::Config;
use crate::statistics::Statistics;
use crate::util::grow_vec_by_n;

#[derive(Clone, Default)]
struct RecvPcb {
	pub seq: IdSequence,
}

#[derive(Debug, Clone, Default)]
struct HeaderBuf {
	inner: [u8; Header::SIZE],
	complete: bool,
}

impl HeaderBuf {
	/// Check whether the buffer holds a complete header.
	fn is_complete(&self) -> bool {
		self.complete
	}

	/// Get the [`ReadInstructions`] to complete the header.
	///
	/// This function marks the header as completed.
	///
	/// # Panics
	///
	/// If it is called when [`HeaderBuf::is_complete()`] is `true`.
	fn get_read_instructions(&mut self) -> ReadInstructions {
		assert!(
			!self.complete,
			"tried to get instructions to read a complete header"
		);

		self.complete = true;
		ReadInstructions { buf: &mut self.inner }
	}

	/// Parse the header out of the buffer.
	///
	/// This function will try to parse a [`Header`] from the buffer. It will
	/// also reset the buffer and mark it as incomplete.
	///
	/// # Panics
	///
	/// If it is called when [`HeaderBuf::is_complete()`] is `false`.
	fn parse_header(
		&mut self,
		pcb: &mut RecvPcb,
		config: &Config,
	) -> Result<Header, VerifyError> {
		assert!(
			self.complete,
			"header must be complete before parsing"
		);

		let with_checksum = if config.verify_header_checksum() {
			WithChecksum::Yes
		} else {
			WithChecksum::No
		};

		let verify_id = if config.verify_packet_order() {
			VerifyId::Yes(&mut pcb.seq)
		} else {
			VerifyId::No
		};

		let bytes = self.inner;
		self.complete = false;

		Header::try_from_bytes(bytes, with_checksum, verify_id)
	}
}

pub struct Deframer {
	header_buf: HeaderBuf,
	last_header: Option<Header>,
	payload_read: bool,
	payload: Vec<u8>,

	config: Config,
	pcb: RecvPcb,
}

/// Instructions for the IO code from the [`Deframer`].
#[derive(Debug)]
pub struct ReadInstructions<'a> {
	pub buf: &'a mut [u8],
}

impl<'a> ReadInstructions<'a> {
	/// Create instructions to read `n` bytes and place them at the end of `vec`.
	pub fn append_to(vec: &'a mut Vec<u8>, n: usize) -> Self {
		Self { buf: grow_vec_by_n(vec, n) }
	}
}

impl Deframer {
	pub fn new(config: Config) -> Self {
		let payload = make_new_payload_buf(&config);

		Self {
			config,
			header_buf: HeaderBuf::default(),
			last_header: None,
			payload_read: false,
			payload,
			pcb: RecvPcb::default(),
		}
	}

	pub fn config(&self) -> &Config {
		&self.config
	}

	/// Take the current payload buffer and replace it with a new one.
	fn take_payload(&mut self) -> Vec<u8> {
		let new_buf = make_new_payload_buf(&self.config);
		mem::replace(&mut self.payload, new_buf)
	}

	/// Reset the internal state of the [`Deframer`].
	///
	/// This function does **NOT** clear the payload.
	fn reset(&mut self) {
		self.last_header = None;
		self.payload_read = false;
	}

	/// Reset the internal state of the [`Deframer`] and clear the payload.
	fn reset_all(&mut self) {
		self.reset();
		self.payload.clear();
	}
}

#[derive(Debug)]
pub enum DeframeStatus<'a, T> {
	Ready(T),
	NotReady(ReadInstructions<'a>),
}

impl Deframer {
	/// Try to deframe a complete payload that may span across multiple packets.
	///
	/// This function will return [`DeframeStatus::Ready`] when either the entire
	/// payload has been read or when an error during parsing of the data occurred.
	/// If the deframer does not have enough data to finish the payload then this
	/// function returns with [`DeframeStatus::NotReady`]. This variant carries
	/// [`ReadInstructions`] that tell the IO code: 1) how much data to read and
	/// 2) where to place that data. The IO code, after reading that data, must
	/// call [`Deframer::deframe()`] again until the deframer becomes ready.
	pub fn deframe(
		&mut self,
		statistics: &mut Statistics,
	) -> DeframeStatus<Result<Vec<u8>, DeframeError>> {
		use DeframeStatus::{NotReady, Ready};

		loop {
			if self.last_header.is_none() {
				if !self.header_buf.is_complete() {
					return NotReady(
						// SAFETY: The panic condition is checked above.
						self.header_buf.get_read_instructions(),
					);
				}

				// SAFETY: `self.header_buf.get_read_instructions()` ensures the
				//         buffer is complete.
				let header = match self
					.header_buf
					.parse_header(&mut self.pcb, &self.config)
				{
					Ok(x) => x,
					Err(e) => {
						self.reset_all();
						return Ready(Err(DeframeError::from(e)));
					},
				};

				self.last_header = Some(header);
			}

			// SAFETY: If execution has reached this point then `self.last_header`
			//         must be `Some(...)`. The above `if` statement checks if
			//         `self.last_header` is `None` and sets it to `Some(...)`.
			let header = self
				.last_header
				.clone()
				.expect("payload is ready but no header exists");

			let payload_length =
				header.length.to_payload_length().as_usize();

			// Read the payload of this packet into `self.payload` if we have not
			// done that.
			if !self.payload_read {
				let total_payload_length = usize::saturating_add(
					self.payload.len(),
					payload_length,
				);

				if let Some(lim) = self.config.max_size {
					if total_payload_length > lim.get() {
						self.reset_all();
						return Ready(Err(
							DeframeError::ExceededMaximumSize,
						));
					}
				}

				self.payload_read = true;

				return NotReady(ReadInstructions::append_to(
					&mut self.payload,
					payload_length,
				));
			}

			self.reset();

			statistics.inc_packets();

			if header.flags.contains(Flags::MORE_DATA) {
				continue;
			}

			return Ready(Ok(self.take_payload()));
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeframeError {
	ChecksumError,
	ExceededMaximumSize,
	InvalidHeader,
	OutOfOrder,
	VersionMismatch,
	ZeroSizeFragment,
}

impl From<VerifyError> for DeframeError {
	fn from(value: VerifyError) -> Self {
		use DeframeError as B;
		use VerifyError as A;

		match value {
			A::InvalidChecksum | A::InvalidLength => B::InvalidHeader,
			A::OutOfOrder => B::OutOfOrder,
			A::VersionMismatch => B::VersionMismatch,
		}
	}
}

fn make_new_payload_buf(config: &Config) -> Vec<u8> {
	match (config.max_size, config.size_estimate) {
		(Some(lim), Some(estimate)) if estimate > lim => {
			Vec::with_capacity(lim.get())
		},
		(_, Some(estimate)) => Vec::with_capacity(estimate.get()),
		(_, None) => Vec::new(),
	}
}
