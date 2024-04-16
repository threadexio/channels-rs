use crate::{
	error::SendError,
	io::{AsyncWrite, Buf, Cursor, Write},
	sender::Config,
	util::StatIO,
};

use super::Pcb;

use channels_packet::{
	header::{Header, WithChecksum},
	Flags, PacketLength, PayloadLength,
};

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

struct Send<'a, W, B>
where
	B: Buf,
{
	config: &'a Config,
	has_sent_one_packet: bool,
	payload: B,
	pcb: &'a mut Pcb,
	writer: &'a mut StatIO<W>,
}

impl<'a, W, B> Send<'a, W, B>
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

pub async fn send<'a, W, B>(
	config: &'a Config,
	pcb: &'a mut Pcb,
	writer: &'a mut StatIO<W>,
	payload: B,
) -> Result<(), SendPayloadError<W::Error>>
where
	W: Write,
	B: Buf,
{
	Send {
		config,
		has_sent_one_packet: false,
		payload,
		pcb,
		writer,
	}.run() await
}

impl<'a, W, B> Send<'a, W, B>
where
	W: Write,
	B: Buf,
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

			let mut header_buf = Cursor::new(header.to_bytes(with_checksum));
			let payload_length = header.length.to_payload_length().as_usize();

			if payload_length == 0 {
				self.writer.write(&mut header_buf) await?;
			} else {
				let payload = self.payload.by_ref().take(payload_length);
				let mut packet = header_buf.chain(payload);

				match payload_length {
					_ if self.config.coalesce_writes() => {
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

		if self.config.flush_on_send() {
			self.writer.flush() await?;
		}

		Ok(())
	}
}

	}
}
