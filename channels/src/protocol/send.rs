use channels_packet::header::{Header, WithChecksum};
use channels_packet::id::IdSequence;
use channels_packet::{Flags, PacketLength, PayloadLength};

use crate::error::SendError;
use crate::io::{AsyncWrite, Write};
use crate::sender::Config;
use crate::statistics::StatIO;

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
pub struct SendPcb {
	pub seq: IdSequence,
}

struct Packet<'a> {
	header: Header,
	payload: &'a [u8],
}

struct PayloadBuf<'a> {
	inner: &'a [u8],
}

impl<'a> PayloadBuf<'a> {
	fn remaining(&self) -> usize {
		self.inner.len()
	}

	fn consume(&mut self, n: usize) -> &'a [u8] {
		let (ret, inner) = self.inner.split_at(n);
		self.inner = inner;
		ret
	}
}

struct AsPackets<'a> {
	payload: PayloadBuf<'a>,
	pcb: &'a mut SendPcb,

	has_one_packet: bool,
}

fn as_packets<'a>(
	pcb: &'a mut SendPcb,
	payload: &'a [u8],
) -> AsPackets<'a> {
	AsPackets {
		pcb,
		has_one_packet: false,
		payload: PayloadBuf { inner: payload },
	}
}

impl<'a> Iterator for AsPackets<'a> {
	type Item = Packet<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		let packet =
			match (self.payload.remaining(), self.has_one_packet) {
				// If there is no more data and we have already sent
				// one packet, then exit.
				(0, true) => return None,
				// If there is no more data and we have not sent any
				// packets, then send one packet with no payload.
				(0, false) => Packet {
					header: Header {
						length: PacketLength::MIN,
						flags: Flags::empty(),
						id: self.pcb.seq.advance(),
					},
					payload: &[],
				},
				// If the remaining data is more than what fits inside
				// one packet, return a header for a full packet.
				(len, _) if len > PayloadLength::MAX.as_usize() => {
					Packet {
						header: Header {
							length: PayloadLength::MAX
								.to_packet_length(),
							flags: Flags::MORE_DATA,
							id: self.pcb.seq.advance(),
						},
						payload: self
							.payload
							.consume(PayloadLength::MAX.as_usize()),
					}
				},
				// If the remaining data is equal or less than what
				// fits inside one packet, return a header for exactly
				// that amount of data.
				(len, _) => Packet {
					header: Header {
						// SAFETY: `len` is less that PayloadLength::MAX and since
						//         `PayloadLength` is a u16, `len` can be cast to
						//         u16 without losses.
						#[allow(clippy::cast_possible_truncation)]
						length: PayloadLength::new(len as u16)
							.expect(
								"len should be a valid PayloadLength",
							)
							.to_packet_length(),
						flags: Flags::empty(),
						id: self.pcb.seq.advance(),
					},
					payload: self.payload.consume(len),
				},
			};

		self.has_one_packet = true;

		Some(packet)
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
	let with_checksum = if config.use_header_checksum() {
		WithChecksum::Yes
	} else {
		WithChecksum::No
	};

	let mut buf = Vec::new();

	for packet in as_packets(pcb, payload) {
		let header_bytes = packet.header.to_bytes(with_checksum);

		if config.coalesce_writes() {
			buf.reserve_exact(packet.header.length.as_usize());
			buf.extend_from_slice(&header_bytes);
			buf.extend_from_slice(packet.payload);
			writer.write(&buf) await?;
			buf.clear();
		} else {
			writer.write(&header_bytes) await?;
			writer.write(packet.payload) await?;
		}

		writer.statistics.inc_packets();
	}

	if config.flush_on_send() {
		writer.flush() await?;
	}

	writer.statistics.inc_ops();

	Ok(())
}

	}
}
