use core::mem;

use alloc::vec::Vec;

use channels_packet::header::{
	Header, VerifyError, VerifyId, WithChecksum,
};
use channels_packet::id::IdSequence;
use channels_packet::Flags;

use crate::error::RecvError;
use crate::io::{
	AsyncRead, AsyncReadExt, BufMut, Cursor, Read, ReadExt,
};
use crate::receiver::Config;
use crate::statistics::{StatIO, Statistics};
use crate::util::grow_vec_by_n;

/// A collection of all things needed to parse a header.
struct RecvPcb {
	seq: IdSequence,
}

/// Instructions for calling code on how much data to read and where to place it.
struct ReadInstructions<'a> {
	// The `+ Send` bound is needed so that `ReadInstructions` is `Send`. This
	// is required for the async version to be able to use it.
	buf: &'a mut (dyn BufMut + Send),
}

/// An enum representing the status of the [`Decoder`].
enum DecodeStatus<'a, T> {
	/// This variant signals that the decoder was able to produce a value.
	Ready(T),
	/// This variant signals that the decoder does not have enough data to produce
	/// a value and wants data read into the buffer specified by the [`ReadInstructions`]
	/// it contains.
	WantsRead(ReadInstructions<'a>),
}

/// Decoder for the channels protocol.
///
/// The decoder is a simple state machine that reads as many packets as needed
/// in order to produce a complete payload. The calling code of the decoder must
/// call [`Decoder::decode()`] repeatedly until it either returns a payload
/// (returned in the form of a [`Vec<u8>`]) or an error. If, at any point,
/// [`Decoder::decode()`] needs more data to continue, signalled by [`DecodeStatus::WantsRead`],
/// the caller must fullfil that request immediately before the decoder can proceed.
struct Decoder {
	header_buf: Cursor<[u8; Header::SIZE]>,
	last_header: Option<Header>,
	payload_buf: Cursor<Vec<u8>>,
	pcb: RecvPcb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DecodeError {
	ChecksumError,
	ExceededMaximumSize,
	InvalidHeader,
	OutOfOrder,
	VersionMismatch,
	ZeroSizeFragment,
}

impl From<VerifyError> for DecodeError {
	fn from(value: VerifyError) -> Self {
		use {DecodeError as B, VerifyError as A};

		match value {
			A::InvalidChecksum => B::ChecksumError,
			A::InvalidLength => B::InvalidHeader,
			A::OutOfOrder => B::OutOfOrder,
			A::VersionMismatch => B::VersionMismatch,
		}
	}
}

impl Decoder {
	fn take_payload(&mut self, config: &Config) -> Vec<u8> {
		self.payload_buf.set_pos(0);
		mem::replace(
			self.payload_buf.get_mut(),
			make_payload_buf(config),
		)
	}

	fn reset(&mut self, config: &Config) {
		self.header_buf.set_pos(0);
		self.last_header = None;
		let _ = self.take_payload(config);
	}

	pub fn decode(
		&mut self,
		config: &Config,
		statistics: &mut Statistics,
	) -> DecodeStatus<'_, Result<Vec<u8>, DecodeError>> {
		use DecodeStatus::{Ready, WantsRead};

		statistics.inc_ops();

		loop {
			if self.last_header.is_none() {
				if self.header_buf.has_remaining_mut() {
					return WantsRead(ReadInstructions {
						buf: &mut self.header_buf,
					});
				}

				self.header_buf.set_pos(0);
				let header = match parse_header(
					*self.header_buf.get_mut(),
					config,
					&mut self.pcb,
				) {
					Ok(x) => x,
					Err(e) => {
						self.reset(config);
						return Ready(Err(DecodeError::from(e)));
					},
				};

				let payload_length =
					header.length.to_payload_length().as_usize();

				if let Some(max_size) = config.max_size {
					let new_payload_length = usize::saturating_add(
						self.payload_buf.get_mut().len(),
						payload_length,
					);

					if new_payload_length > max_size.get() {
						self.reset(config);
						return Ready(Err(
							DecodeError::ExceededMaximumSize,
						));
					}
				}

				if header.flags.contains(Flags::MORE_DATA)
					&& payload_length == 0
				{
					self.reset(config);
					return Ready(Err(DecodeError::ZeroSizeFragment));
				}

				grow_vec_by_n(
					self.payload_buf.get_mut(),
					payload_length,
				);
				self.last_header = Some(header);
			}

			let header = self
				.last_header
				.clone()
				.expect("header should have been set before");

			if self.payload_buf.has_remaining_mut() {
				return WantsRead(ReadInstructions {
					buf: &mut self.payload_buf,
				});
			}

			statistics.inc_packets();

			if header.flags.contains(Flags::MORE_DATA) {
				self.last_header = None;
				continue;
			}

			let payload = self.take_payload(config);

			self.reset(config);
			return Ready(Ok(payload));
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreRecvError<Io> {
	ChecksumError,
	ExceededMaximumSize,
	InvalidHeader,
	Io(Io),
	OutOfOrder,
	VersionMismatch,
	ZeroSizeFragment,
}

impl<Io> From<DecodeError> for CoreRecvError<Io> {
	fn from(value: DecodeError) -> Self {
		use CoreRecvError as B;
		use DecodeError as A;

		match value {
			A::ChecksumError => B::ChecksumError,
			A::ExceededMaximumSize => B::ExceededMaximumSize,
			A::InvalidHeader => B::InvalidHeader,
			A::OutOfOrder => B::OutOfOrder,
			A::VersionMismatch => B::VersionMismatch,
			A::ZeroSizeFragment => B::ZeroSizeFragment,
		}
	}
}

impl<Des, Io> From<CoreRecvError<Io>> for RecvError<Des, Io> {
	fn from(value: CoreRecvError<Io>) -> Self {
		use CoreRecvError as A;
		use RecvError as B;

		match value {
			A::ChecksumError => B::ChecksumError,
			A::ExceededMaximumSize => B::ExceededMaximumSize,
			A::InvalidHeader => B::InvalidHeader,
			A::Io(x) => B::Io(x),
			A::OutOfOrder => B::OutOfOrder,
			A::VersionMismatch => B::VersionMismatch,
			A::ZeroSizeFragment => B::ZeroSizeFragment,
		}
	}
}

pub(crate) struct ReceiverCore<R> {
	pub(crate) reader: StatIO<R>,
	pub(crate) config: Config,
	decoder: Decoder,
}

impl<R> ReceiverCore<R> {
	pub fn new(reader: StatIO<R>, config: Config) -> Self {
		let decoder = Decoder {
			header_buf: Cursor::new([0u8; Header::SIZE]),
			last_header: None,
			payload_buf: Cursor::new(make_payload_buf(&config)),
			pcb: RecvPcb { seq: IdSequence::default() },
		};

		Self { reader, config, decoder }
	}
}

channels_macros::replace! {
	replace: {
		// Synchronous version
		[
			(async =>)
			(await =>)
			(recv  => recv_sync)
			(Read  => Read)
		]
		// Asynchronous version
		[
			(async => async)
			(await => .await)
			(recv => recv_async)
			(Read => AsyncRead + Unpin)
		]
	}
	code: {

impl<R> ReceiverCore<R>
where
	R: Read,
{
	pub async fn recv(
		&mut self,
	) -> Result<Vec<u8>, CoreRecvError<R::Error>> {
		use DecodeStatus::{Ready, WantsRead};

		loop {
			match self.decoder.decode(&self.config, &mut self.reader.statistics) {
				Ready(Ok(payload)) => return Ok(payload),
				Ready(Err(e)) => return Err(CoreRecvError::from(e)),
				WantsRead(instructions) => {
					self.reader.read(instructions.buf) await
						.map_err(CoreRecvError::Io)?;
				}
			}
		}
	}
}

	}
}

fn make_payload_buf(config: &Config) -> Vec<u8> {
	match (config.size_estimate, config.max_size) {
		(Some(estimate), Some(max)) if estimate > max => {
			Vec::with_capacity(max.get())
		},
		(Some(estimate), _) => Vec::with_capacity(estimate.get()),
		(None, _) => Vec::new(),
	}
}

fn parse_header(
	buf: [u8; Header::SIZE],
	config: &Config,
	pcb: &mut RecvPcb,
) -> Result<Header, VerifyError> {
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

	Header::try_from_bytes(buf, with_checksum, verify_id)
}
