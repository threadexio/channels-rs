use channels_packet::{
	Flags, Header, IdGenerator, PacketLength, PayloadLength,
};

use crate::error::VerifyError;
use crate::io::{
	AsyncRead, AsyncWrite, Buf, Contiguous, Cursor, Read, Write,
};
use crate::util::StatIO;

#[derive(Clone)]
pub struct Pcb {
	id_gen: IdGenerator,
}

impl Pcb {
	pub const fn new() -> Self {
		Self { id_gen: IdGenerator::new() }
	}
}

struct SendPayload<'a, W, B> {
	pcb: &'a mut Pcb,
	writer: &'a mut StatIO<W>,
	payload: B,
	has_sent_one_packet: bool,
}

impl<'a, W, B> SendPayload<'a, W, B> {
	pub fn new(
		pcb: &'a mut Pcb,
		writer: &'a mut StatIO<W>,
		payload: B,
	) -> Self {
		Self { pcb, writer, payload, has_sent_one_packet: false }
	}
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
				flags: Flags::zero(),
				id: self.pcb.id_gen.next_id(),
			}),
			// If the remaining data is more than what fits inside
			// one packet, return a header for a full packet.
			(rem, _) if rem > PayloadLength::MAX.as_usize() => {
				Some(Header {
					length: PayloadLength::MAX.to_packet_length(),
					flags: Flags::MORE_DATA,
					id: self.pcb.id_gen.next_id(),
				})
			},
			// If the remaining data is equal or less than what
			// fits inside one packet, return a header for exactly
			// that amount of data.
			#[allow(clippy::cast_possible_truncation)]
			(rem, _) => Some(Header {
				length: PayloadLength::new_saturating(rem as u16)
					.to_packet_length(),
				flags: Flags::zero(),
				id: self.pcb.id_gen.next_id(),
			}),
		}
	}
}

#[derive(Debug)]
pub enum RecvPayloadError<Io> {
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
		}
	}
}

struct RecvPayload<'a, R> {
	pcb: &'a mut Pcb,
	reader: &'a mut StatIO<R>,
}

impl<'a, R> RecvPayload<'a, R> {
	pub fn new(pcb: &'a mut Pcb, reader: &'a mut StatIO<R>) -> Self {
		Self { pcb, reader }
	}
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
	pcb: &'a mut Pcb,
	writer: &'a mut StatIO<W>,
	payload: B,
) -> Result<(), W::Error>
where
	W: Write,
	B: Buf,
{
	SendPayload::new(pcb, writer, payload).run() await
}

impl<'a, W, B> SendPayload<'a, W, B>
where
	W: Write,
	B: Buf,
{
	pub async fn run(mut self) -> Result<(), W::Error> {
		while let Some(header) = self.next_header() {
			self.has_sent_one_packet = true;

			let mut header_buf = Cursor::new(header.to_bytes());

			match header.length.to_payload_length().as_usize() {
				0 => {
					self.writer.write(&mut header_buf) await?;
				}
				payload_length => {
					let payload = self.payload.by_ref().take(payload_length);
					let packet = header_buf.chain(payload);
					let mut packet = packet.copy_to_contiguous();
					self.writer.write(&mut packet) await?;
				}
			};

			#[cfg(feature = "statistics")]
			self.writer.statistics.inc_packets();
		}

		#[cfg(feature = "statistics")]
		self.writer.statistics.inc_ops();

		Ok(())
	}
}

pub async fn recv<'a, R>(
	pcb: &'a mut Pcb,
	reader: &'a mut StatIO<R>,
) -> Result<impl Contiguous, RecvPayloadError<R::Error>>
where
	R: Read,
{
	RecvPayload::new(pcb, reader).run() await
}

impl<'a, R> RecvPayload<'a, R>
where
	R: Read,
{
	pub async fn run(
		self,
	) -> Result<impl Contiguous, RecvPayloadError<R::Error>> {
		let mut full_payload = alloc::vec::Vec::new();

		loop {
			let mut header = [0u8; Header::SIZE];
			self.reader
				.read(&mut header[..]) await
				.map_err(RecvPayloadError::Io)?;

			let header =
				Header::read_from(&header, &mut self.pcb.id_gen)
					.map_err(VerifyError::from)
					.map_err(RecvPayloadError::Verify)?;

			let payload_start = full_payload.len();
			let payload_length =
				header.length.to_payload_length().as_usize();

			full_payload.reserve_exact(payload_length);
			full_payload.resize(payload_start + payload_length, 0);

			self.reader
				.read(&mut full_payload[payload_start..]) await
				.map_err(RecvPayloadError::Io)?;

			if !(header.flags & Flags::MORE_DATA) {
				break;
			}
		}

		Ok(Cursor::new(full_payload))
	}
}

	}
}
