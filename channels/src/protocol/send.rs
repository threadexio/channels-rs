use core::mem;

use alloc::vec::Vec;

use channels_packet::header::{Header, WithChecksum};
use channels_packet::id::IdSequence;
use channels_packet::{Flags, PacketLength, PayloadLength};

use crate::error::SendError;
use crate::io::{AsyncWrite, Write};
use crate::sender::Config;
use crate::statistics::StatIO;

pub(crate) struct SenderCore<W> {
	pub(crate) writer: StatIO<W>,
	pub(crate) config: Config,
	pcb: SendPcb,
	write_buf: Vec<u8>,
}

impl<W> SenderCore<W> {
	pub fn new(writer: StatIO<W>, config: Config) -> Self {
		let write_buf = Vec::new();
		let pcb = SendPcb::default();

		Self { writer, config, pcb, write_buf }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreSendError<Io> {
	Io(Io),
}

impl<Io> From<Io> for CoreSendError<Io> {
	fn from(value: Io) -> Self {
		Self::Io(value)
	}
}

impl<Ser, Io> From<CoreSendError<Io>> for SendError<Ser, Io> {
	fn from(value: CoreSendError<Io>) -> Self {
		use CoreSendError as A;
		use SendError as B;

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

/// Estimate the total size of all of the packets needed to hold `payload`.
fn estimate_total_size(payload_len: usize) -> usize {
	let n_full_packets = payload_len / PayloadLength::MAX.as_usize();
	let rem_bytes = payload_len % PayloadLength::MAX.as_usize();

	let mut size = n_full_packets * PacketLength::MAX.as_usize();

	if rem_bytes != 0 || n_full_packets == 0 {
		// SAFETY: `rem` is the result of a modulo operation with `PayloadLength::MAX`.
		//         The divisor is less than `u16::MAX`, so the result must also be
		//         less than `u16::MAX`. Casting to u16 is safe.
		#[allow(clippy::cast_possible_truncation)]
		let rem = rem_bytes as u16;

		let rem = PayloadLength::new(rem)
			.expect("rem should be smaller than PayloadLength::MAX");

		size += rem.to_packet_length().as_usize();
	}

	size
}

channels_macros::replace! {
	replace: {
		// Synchronous version
		[
			(async =>)
			(await =>)
			(send  => send_sync)
			(Write => Write)
		]
		// Asynchronous version
		[
			(async => async)
			(await => .await)
			(send => send_async)
			(Write => AsyncWrite)
		]
	}
	code: {

impl<W> SenderCore<W>
where
	W: Write,
{
	pub async fn send(
		&mut self,
		data: &[u8],
	) -> Result<(), CoreSendError<W::Error>> {
		let with_checksum = if self.config.use_header_checksum() {
			WithChecksum::Yes
		} else {
			WithChecksum::No
		};

		if self.config.coalesce_writes() {
			let estimated_size = estimate_total_size(data.len());
			self.write_buf.clear();
			self.write_buf.reserve(estimated_size);
		}

		for packet in as_packets(&mut self.pcb, data) {
			let header_bytes = packet.header.to_bytes(with_checksum);

			if self.config.coalesce_writes() {
				self.write_buf.extend_from_slice(&header_bytes);
				self.write_buf.extend_from_slice(packet.payload);
			} else {
				self.writer.write(&header_bytes) await?;
				self.writer.write(packet.payload) await?;
			}

			self.writer.statistics.inc_packets();
		}

		if self.config.coalesce_writes() {
			self.writer.write(&self.write_buf) await?;

			if !self.config.keep_write_allocations() {
				let _ = mem::take(&mut self.write_buf);
			}
		}

		if self.config.flush_on_send() {
			self.writer.flush() await?;
		}

		self.writer.statistics.inc_ops();

		Ok(())
	}
}

	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_estimate_total_size() {
		#[derive(Clone, Copy)]
		struct TestVector {
			id: &'static str,
			input: usize,
			expected: usize,
		}

		static TEST_VECTORS: &[TestVector] = &[
			TestVector {
				id: "no payload",
				input: 0,
				expected: PacketLength::MIN.as_usize(),
			},
			TestVector {
				id: "1 byte payload",
				input: 1,
				expected: PayloadLength::new_saturating(1)
					.to_packet_length()
					.as_usize(),
			},
			TestVector {
				id: "full packet - 1",
				input: PayloadLength::MAX.as_usize() - 1,
				expected: PayloadLength::new_saturating(
					PayloadLength::MAX.as_u16() - 1,
				)
				.to_packet_length()
				.as_usize(),
			},
			TestVector {
				id: "full packet",
				input: PayloadLength::MAX.as_usize(),
				expected: PacketLength::MAX.as_usize(),
			},
			TestVector {
				id: "full packet + 1",
				input: PayloadLength::MAX.as_usize() + 1,
				expected: PacketLength::MAX.as_usize()
					+ PayloadLength::new_saturating(1)
						.to_packet_length()
						.as_usize(),
			},
			TestVector {
				id: "2 full packets - 1",
				input: 2 * PayloadLength::MAX.as_usize() - 1,
				expected: PacketLength::MAX.as_usize()
					+ PayloadLength::new_saturating(
						PayloadLength::MAX.as_u16() - 1,
					)
					.to_packet_length()
					.as_usize(),
			},
			TestVector {
				id: "2 full packets",
				input: 2 * PayloadLength::MAX.as_usize(),
				expected: 2 * PacketLength::MAX.as_usize(),
			},
			TestVector {
				id: "2 full packets + 1",
				input: 2 * PayloadLength::MAX.as_usize() + 1,
				expected: 2 * PacketLength::MAX.as_usize()
					+ PayloadLength::new_saturating(1)
						.to_packet_length()
						.as_usize(),
			},
		];

		fn test(test_case: TestVector) {
			assert_eq!(
				estimate_total_size(test_case.input),
				test_case.expected,
				"test case \"{}\" failed",
				test_case.id
			);
		}

		TEST_VECTORS.iter().copied().for_each(test);
	}
}
