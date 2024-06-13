use alloc::vec::Vec;

use channels_packet::header::{
	Header, VerifyError, VerifyId, WithChecksum,
};
use channels_packet::id::IdSequence;
use channels_packet::Flags;

use crate::error::RecvError;
use crate::io::{AsyncRead, Read};
use crate::receiver::Config;
use crate::statistics::StatIO;
use crate::util::grow_vec_by_n;

pub(crate) struct ReceiverCore<R> {
	pub(crate) reader: StatIO<R>,
	pub(crate) config: Config,

	pcb: RecvPcb,
	recv_buf: Vec<u8>,
}

impl<R> ReceiverCore<R> {
	pub fn new(reader: StatIO<R>, config: Config) -> Self {
		let pcb = RecvPcb::default();

		let recv_buf = match (config.size_estimate, config.max_size) {
			(Some(estimate), Some(max)) if estimate > max => {
				Vec::with_capacity(max.get())
			},
			(Some(estimate), _) => Vec::with_capacity(estimate.get()),
			(None, _) => Vec::new(),
		};

		Self { reader, config, pcb, recv_buf }
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

impl<Io> From<VerifyError> for CoreRecvError<Io> {
	fn from(value: VerifyError) -> Self {
		use CoreRecvError as B;
		use VerifyError as A;

		match value {
			A::InvalidChecksum => B::ChecksumError,
			A::InvalidLength => B::InvalidHeader,
			A::OutOfOrder => B::OutOfOrder,
			A::VersionMismatch => B::VersionMismatch,
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

#[derive(Clone, Default)]
pub struct RecvPcb {
	pub seq: IdSequence,
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
			(Read => AsyncRead)
		]
	}
	code: {

impl<R> ReceiverCore<R>
where
	R: Read,
{
	pub async fn recv(
		&mut self,
	) -> Result<&mut [u8], CoreRecvError<R::Error>> {

		let Self {
			config, pcb, reader, recv_buf
		} = self;

		reader.statistics.inc_ops();

		recv_buf.clear();

		loop {
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

			let mut header_buf = [0u8; Header::SIZE];
			reader.read(&mut header_buf) await.map_err(CoreRecvError::Io)?;

			let header = Header::try_from_bytes(header_buf, with_checksum, verify_id)?;

			let payload_buf = grow_vec_by_n(recv_buf, header.length.to_payload_length().as_usize());
			reader.read(payload_buf) await.map_err(CoreRecvError::Io)?;

			reader.statistics.inc_packets();

			if !header.flags.contains(Flags::MORE_DATA) {
				break;
			}
		}

		Ok(recv_buf)
	}
}

	}
}
