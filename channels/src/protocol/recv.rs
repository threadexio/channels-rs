use crate::{
	error::RecvError,
	io::{AsyncRead, Read},
	receiver::Config,
	util::StatIO,
};

use super::Pcb;

use channels_packet::{
	header::{Header, VerifyError, VerifyId, WithChecksum},
	Flags, PacketLength,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecvPayloadError<Io> {
	ChecksumError,
	ExceededMaximumSize,
	InvalidHeader,
	Io(Io),
	OutOfOrder,
	VersionMismatch,
	ZeroSizeFragment,
}

impl<Io> From<Io> for RecvPayloadError<Io> {
	fn from(value: Io) -> Self {
		Self::Io(value)
	}
}

impl<Io> RecvPayloadError<Io> {
	pub fn from_verify_error(value: VerifyError) -> Self {
		use RecvPayloadError as B;
		use VerifyError as A;

		match value {
			A::InvalidChecksum => B::ChecksumError,
			A::InvalidLength => B::InvalidHeader,
			A::OutOfOrder => B::OutOfOrder,
			A::VersionMismatch => B::VersionMismatch,
		}
	}
}

impl<Des, Io> From<RecvPayloadError<Io>> for RecvError<Des, Io> {
	fn from(value: RecvPayloadError<Io>) -> Self {
		use RecvError as B;
		use RecvPayloadError as A;

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

struct Recv<'a, R> {
	config: &'a Config,
	pcb: &'a mut Pcb,
	reader: &'a mut StatIO<R>,
}

channels_macros::replace! {
	replace: {
		// Synchronous version
		[
			(async =>)
			(await =>)
			(recv  => recv_sync)
			(Read  => Read)
			(run   => run_sync)
		]
		// Asynchronous version
		[
			(async => async)
			(await => .await)
			(recv => recv_async)
			(Read => AsyncRead)
			(run => run_async)
		]
	}
	code: {

pub async fn recv<R>(
	config: &Config,
	pcb: &mut Pcb,
	reader: &mut StatIO<R>,
) -> Result<Vec<u8>, RecvPayloadError<R::Error>>
where
	R: Read,
{
	Recv {
		config,
		pcb,
		reader,
	}.run() await
}

impl<'a, R> Recv<'a, R>
where
	R: Read,
{
	pub async fn run(
		self,
	) -> Result<Vec<u8>, RecvPayloadError<R::Error>> {
		use alloc::vec::Vec;

		let mut full_payload = match (self.config.size_estimate, self.config.max_size) {
			(Some(estimate), Some(max_size)) if max_size < estimate => Vec::with_capacity(max_size.get()),
			(Some(estimate), _) => Vec::with_capacity(estimate.get()),
			(None, _) => Vec::new()
		};

		loop {
			let mut header = [0u8; Header::SIZE];
			self.reader.read(&mut header[..]) await?;

			let with_checksum =
				if self.config.verify_header_checksum {
					WithChecksum::Yes
				} else {
					WithChecksum::No
				};

			let header =
				Header::try_from_bytes(header, with_checksum, VerifyId::Yes(&mut self.pcb.seq))
					.map_err(RecvPayloadError::from_verify_error)?;

			if header.length == PacketLength::MIN
				&& header.flags.contains(Flags::MORE_DATA)
			{
				return Err(RecvPayloadError::ZeroSizeFragment);
			}

			let payload_start = full_payload.len();
			let payload_length =
				header.length.to_payload_length().as_usize();
			let payload_buf_new_len = payload_start + payload_length;

			if let Some(max_size) = self.config.max_size {
				if payload_buf_new_len > max_size.get() {
					return Err(RecvPayloadError::ExceededMaximumSize);
				}
			}

			full_payload.reserve_exact(payload_length);
			full_payload.resize(payload_buf_new_len, 0);

			self.reader.read(&mut full_payload[payload_start..]) await?;

			if !header.flags.contains(Flags::MORE_DATA) {
				break;
			}
		}

		Ok(full_payload)
	}
}

	}
}
