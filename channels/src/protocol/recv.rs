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

pub(crate) struct ReceiverCore<R> {
	pub(crate) reader: StatIO<R>,
	pub(crate) config: Config,
	state: RecvStateMachine,
}

impl<R> ReceiverCore<R> {
	pub fn new(reader: StatIO<R>, config: Config) -> Self {
		let state = RecvStateMachine {
			pcb: RecvPcb::default(),
			header: None,
			header_buf: Cursor::new([0u8; Header::SIZE]),
			payload: Cursor::new(make_payload_buf(&config)),
		};

		Self { reader, config, state }
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

impl<Io> From<AdvanceError> for CoreRecvError<Io> {
	fn from(value: AdvanceError) -> Self {
		use AdvanceError as A;
		use CoreRecvError as B;

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

/// A collection of all state needed to parse a header.
#[derive(Default)]
struct RecvPcb {
	seq: IdSequence,
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
		use Status::{Ready, WantsRead};

		loop {
			match self.state.advance(&self.config, &mut self.reader.statistics) {
				Ready(x) => {
					self.state.reset();
					return x.map_err(CoreRecvError::from)
				},
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

struct ReadInstructions<'a> {
	buf: &'a mut (dyn BufMut + Send),
}

enum Status<'a, T> {
	Ready(T),
	WantsRead(ReadInstructions<'a>),
}

struct RecvStateMachine {
	pcb: RecvPcb,
	header_buf: Cursor<[u8; Header::SIZE]>,
	header: Option<Header>,
	payload: Cursor<Vec<u8>>,
}

enum AdvanceError {
	ChecksumError,
	ExceededMaximumSize,
	InvalidHeader,
	OutOfOrder,
	VersionMismatch,
	ZeroSizeFragment,
}

impl From<VerifyError> for AdvanceError {
	fn from(value: VerifyError) -> Self {
		use {AdvanceError as B, VerifyError as A};

		match value {
			A::InvalidChecksum => B::ChecksumError,
			A::InvalidLength => B::InvalidHeader,
			A::OutOfOrder => B::OutOfOrder,
			A::VersionMismatch => B::VersionMismatch,
		}
	}
}

impl RecvStateMachine {
	pub fn reset(&mut self) {
		self.header = None;
		self.header_buf.set_pos(0);
		self.payload.set_pos(0);
		self.payload.get_mut().clear();
	}

	pub fn advance(
		&mut self,
		config: &Config,
		statistics: &mut Statistics,
	) -> Status<'_, Result<Vec<u8>, AdvanceError>> {
		use Status::{Ready, WantsRead};

		loop {
			if self.header.is_none() {
				if self.header_buf.has_remaining_mut() {
					return WantsRead(ReadInstructions {
						buf: &mut self.header_buf,
					});
				}

				let header = match parse_header(
					*self.header_buf.get_mut(),
					config,
					&mut self.pcb,
				) {
					Ok(x) => x,
					Err(e) => {
						return Ready(Err(AdvanceError::from(e)));
					},
				};

				let payload_length =
					header.length.to_payload_length().as_usize();

				if let Some(max_size) = config.max_size {
					let new_payload_length = usize::saturating_add(
						self.payload.get_mut().len(),
						payload_length,
					);

					if new_payload_length > max_size.get() {
						return Ready(Err(
							AdvanceError::ExceededMaximumSize,
						));
					}
				}

				if header.flags.contains(Flags::MORE_DATA)
					&& payload_length == 0
				{
					return Ready(Err(
						AdvanceError::ZeroSizeFragment,
					));
				}

				grow_vec_by_n(self.payload.get_mut(), payload_length);
				self.header = Some(header);
			}

			let header = self
				.header
				.clone()
				.expect("header should have been set before");

			if self.payload.has_remaining_mut() {
				return WantsRead(ReadInstructions {
					buf: &mut self.payload,
				});
			}

			statistics.inc_packets();

			if header.flags.contains(Flags::MORE_DATA) {
				self.header = None;
				self.header_buf.set_pos(0);
			} else {
				let payload = mem::replace(
					self.payload.get_mut(),
					make_payload_buf(config),
				);

				statistics.inc_ops();

				return Ready(Ok(payload));
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
