use channels_packet::header::{Header, WithChecksum};
use channels_packet::{Flags, PacketLength, PayloadLength};

use crate::error::SendError;
use crate::io::{AsyncWrite, Write};
use crate::sender::Config;
use crate::util::StatIO;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SendPayloadError<Io> {
	Io(Io),
}

impl<Io> From<Io> for SendPayloadError<Io> {
	fn from(value: Io) -> Self {
		Self::Io(value)
	}
}

impl<Ser, Io> From<SendPayloadError<Io>> for SendError<Ser, Io> {
	fn from(value: SendPayloadError<Io>) -> Self {
		use SendError as B;
		use SendPayloadError as A;

		match value {
			A::Io(x) => B::Io(x),
		}
	}
}

#[derive(Clone, Default)]
pub struct State {}

pub type SendPcb = super::Pcb<State>;

struct SendPayload<'a, 'p, W> {
	config: &'a Config,
	has_sent_one_packet: bool,
	payload: &'p [u8],
	pcb: &'a mut SendPcb,
	writer: &'a mut StatIO<W>,
}

impl<'a, 'p, W> SendPayload<'a, 'p, W> {
	/// Get the header for the next packet.
	///
	/// Returns [`Some`] with the header of the next packet that should be
	/// sent or [`None`] if no packet should be sent.
	fn next_header(&mut self) -> Option<Header> {
		match (self.payload.len(), self.has_sent_one_packet) {
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

channels_macros::replace! {
	replace: {
		// Synchronous version
		[
			(async =>)
			(await =>)
			(send  => send_sync)
			(Write => Write)
			(run   => run_sync)
		]
		// Asynchronous version
		[
			(async => async)
			(await => .await)
			(send => send_async)
			(Write => AsyncWrite)
			(run => run_async)
		]
	}
	code: {

pub async fn send<W>(
	config: &Config,
	pcb: &mut SendPcb,
	writer: &mut StatIO<W>,
	payload: &[u8],
) -> Result<(), SendPayloadError<W::Error>>
where
	W: Write
{
	SendPayload {
		config,
		has_sent_one_packet: false,
		payload,
		pcb,
		writer,
	}.run() await
}

impl<'a, 'b, W> SendPayload<'a, 'b, W>
where
	W: Write,
{
	pub async fn run(mut self) -> Result<(), SendPayloadError<W::Error>> {
		while let Some(header) = self.next_header() {
			self.has_sent_one_packet = true;

			let with_checksum =
				if self.config.use_header_checksum() {
					WithChecksum::Yes
				} else {
					WithChecksum::No
				};

			let payload_length = header.length.to_payload_length().as_usize();
			let header = header.to_bytes(with_checksum);

			if payload_length == 0 {
				self.writer.write(&header) await?;
			} else {
				let payload = &self.payload[..payload_length];
				self.payload = &self.payload[payload_length..];


				if self.config.coalesce_writes() {
					let packet = [&header, payload].concat();
					self.writer.write(&packet) await?;
				} else {
					self.writer.write(&header) await?;
					self.writer.write(payload) await?;
				}
			}

			#[cfg(feature = "statistics")]
			self.writer.statistics.inc_packets();
		}

		#[cfg(feature = "statistics")]
		self.writer.statistics.inc_ops();

		if self.config.flush_on_send() {
			self.writer.flush() await?;
		}

		Ok(())
	}
}

	}
}
