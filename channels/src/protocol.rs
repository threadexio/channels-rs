use channels_packet::{
	header::{Header, VerifyId, WithChecksum},
	id::IdSequence,
	Flags, PacketLength, PayloadLength,
};

use crate::error::{ProtocolError, VerifyError};
use crate::io::{
	chain, AsyncRead, AsyncWrite, Buf, ContiguousMut, Cursor, Read,
	Write,
};
use crate::receiver::Config as RecvConfig;
use crate::sender::Config as SendConfig;
use crate::util::StatIO;

#[derive(Clone)]
pub struct Pcb {
	seq: IdSequence,
}

impl Pcb {
	pub fn new() -> Self {
		Self { seq: IdSequence::new() }
	}
}

struct SendPayload<'a, W, B> {
	pcb: &'a mut Pcb,
	writer: &'a mut StatIO<W>,
	payload: B,
	has_sent_one_packet: bool,
	config: &'a SendConfig,
}

impl<'a, W, B> SendPayload<'a, W, B>
where
	B: Buf,
{
	/// Get the header for the next packet.
	///
	/// Returns [`Some`] with the header of the next packet that should be
	/// sent or [`None`] if no packet should be sent.
	fn next_header(&mut self) -> Option<Header> {
		match (self.payload.remaining(), self.has_sent_one_packet) {
			// If there is no more data and we have already sent
			// one packet, then exit.
			(0, true) => None,
			// If there is no more data and we have not sent any
			// packets, then send one packet with no payload.
			(0, false) => Some(Header {
				length: PacketLength::MIN,
				flags: Flags::empty(),
				id: self.pcb.seq.advance(),
			}),
			// If the remaining data is more than what fits inside
			// one packet, return a header for a full packet.
			(rem, _) if rem > PayloadLength::MAX.as_usize() => {
				Some(Header {
					length: PayloadLength::MAX.to_packet_length(),
					flags: Flags::MORE_DATA,
					id: self.pcb.seq.advance(),
				})
			},
			// If the remaining data is equal or less than what
			// fits inside one packet, return a header for exactly
			// that amount of data.
			#[allow(clippy::cast_possible_truncation)]
			(rem, _) => Some(Header {
				length: PayloadLength::new_saturating(rem as u16)
					.to_packet_length(),
				flags: Flags::empty(),
				id: self.pcb.seq.advance(),
			}),
		}
	}
}

#[derive(Debug)]
pub enum RecvPayloadError<Io> {
	Protocol(ProtocolError),
	Verify(VerifyError),
	Io(Io),
}

impl<Ser, Io> From<RecvPayloadError<Io>>
	for crate::error::RecvError<Ser, Io>
{
	fn from(value: RecvPayloadError<Io>) -> Self {
		use RecvPayloadError as A;
		match value {
			A::Io(v) => Self::Io(v),
			A::Verify(v) => Self::Verify(v),
			A::Protocol(v) => Self::Protocol(v),
		}
	}
}

struct RecvPayload<'a, R> {
	pcb: &'a mut Pcb,
	reader: &'a mut StatIO<R>,
	config: &'a RecvConfig,
}

channels_macros::replace! {
	replace: {
		// Synchronous version
		[
			(async =>)
			(await =>)
			(send  => send_sync)
			(recv  => recv_sync)
			(Write => Write)
			(Read  => Read)
			(run   => run_sync)
		]
		// Asynchronous version
		[
			(async => async)
			(await => .await)
			(send => send_async)
			(recv => recv_async)
			(Write => AsyncWrite)
			(Read => AsyncRead)
			(run => run_async)
		]
	}
	code: {

pub async fn send<'a, W, B>(
	config: &'a SendConfig,
	pcb: &'a mut Pcb,
	writer: &'a mut StatIO<W>,
	payload: B,
) -> Result<(), W::Error>
where
	W: Write,
	B: Buf,
{
	SendPayload {
		pcb,
		writer,
		payload,
		has_sent_one_packet: false,
		config
	}.run() await
}

pub async fn recv<'a, R>(
	config: &'a RecvConfig,
	pcb: &'a mut Pcb,
	reader: &'a mut StatIO<R>,
) -> Result<impl ContiguousMut, RecvPayloadError<R::Error>>
where
	R: Read,
{
	RecvPayload {
		pcb,
		reader,
		config
	}.run() await
}

impl<'a, W, B> SendPayload<'a, W, B>
where
	W: Write,
	B: Buf,
{
	pub async fn run(mut self) -> Result<(), W::Error> {
		while let Some(header) = self.next_header() {
			self.has_sent_one_packet = true;

			let with_checksum =
				if self.config.use_header_checksum {
					WithChecksum::Yes
				} else {
					WithChecksum::No
				};

			let mut header_buf = Cursor::new(header.to_bytes(with_checksum));
			let payload_length = header.length.to_payload_length().as_usize();

			if payload_length == 0 {
				self.writer.write(&mut header_buf) await?;
			} else {
				let payload = self.payload.by_ref().take(payload_length);
				let mut packet = chain(header_buf, payload);

				match payload_length {
					_ if self.config.coalesce_writes => {
						let mut packet = packet.copy_to_contiguous();
						self.writer.write(&mut packet) await?;
					}
					_ => {
						while packet.has_remaining() {
							let chunk = packet.chunk();
							self.writer.write(chunk) await?;
							packet.advance(chunk.len());
						}
					}
				}
			}

			#[cfg(feature = "statistics")]
			self.writer.statistics.inc_packets();
		}

		#[cfg(feature = "statistics")]
		self.writer.statistics.inc_ops();

		if self.config.flush_on_send {
			self.writer.flush() await?;
		}

		Ok(())
	}
}

impl<'a, R> RecvPayload<'a, R>
where
	R: Read,
{
	pub async fn run(
		self,
	) -> Result<impl ContiguousMut, RecvPayloadError<R::Error>> {
		use alloc::vec::Vec;

		let mut full_payload = match (self.config.size_estimate, self.config.max_size) {
			(Some(estimate), max_size) if max_size < estimate.get() => Vec::with_capacity(max_size),
			(Some(estimate), _) => Vec::with_capacity(estimate.get()),
			(None, _) => Vec::new()
		};

		loop {
			let mut header = [0u8; Header::SIZE];
			self.reader
				.read(&mut header[..]) await
				.map_err(RecvPayloadError::Io)?;

			let with_checksum =
				if self.config.verify_header_checksum {
					WithChecksum::Yes
				} else {
					WithChecksum::No
				};

			let header =
				Header::try_from_bytes(header, with_checksum, VerifyId::Yes(&mut self.pcb.seq))
					.map_err(VerifyError::from)
					.map_err(RecvPayloadError::Verify)?;

			let payload_start = full_payload.len();
			let payload_length =
				header.length.to_payload_length().as_usize();
			let payload_buf_new_len = payload_start + payload_length;

			if payload_buf_new_len > self.config.max_size {
				return Err(RecvPayloadError::Protocol(ProtocolError::ExceededMaximumSize));
			}

			full_payload.reserve_exact(payload_length);
			full_payload.resize(payload_buf_new_len, 0);

			self.reader
				.read(&mut full_payload[payload_start..]) await
				.map_err(RecvPayloadError::Io)?;

			if !header.flags.contains(Flags::MORE_DATA) {
				break;
			}
		}

		Ok(Cursor::new(full_payload))
	}
}

	}
}
