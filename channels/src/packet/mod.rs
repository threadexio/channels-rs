pub mod consts {
	pub const MAX_PACKET_SIZE: usize = 0xffff;

	pub const HEADER_SIZE: usize =
		super::header::private::HEADER_SIZE;
	pub const MAX_PAYLOAD_SIZE: usize = MAX_PACKET_SIZE - HEADER_SIZE;
}

pub mod header;
pub mod list;
pub mod types;

use core::iter::Peekable;

use header::Header;
use list::Packet;
use types::{Flags, Id};

#[derive(Debug, Default)]
pub struct Pcb {
	pub id: Id,
}

impl Pcb {
	pub fn finalize<'a, I>(
		&'a mut self,
		packets: I,
	) -> Finalize<'a, I>
	where
		I: Iterator<Item = &'a mut Packet>,
	{
		Finalize { iter: packets.peekable(), pcb: self }
	}
}

pub struct Finalize<'a, I>
where
	I: Iterator<Item = &'a mut Packet>,
{
	iter: Peekable<I>,
	pcb: &'a mut Pcb,
}

impl<'a, I> Iterator for Finalize<'a, I>
where
	I: Iterator<Item = &'a mut Packet>,
{
	type Item = &'a [u8];

	fn next(&mut self) -> Option<Self::Item> {
		let packet = self.iter.next()?;

		let mut header = Header {
			length: packet.write_pos().to_packet_length(),
			flags: Flags::zero(),
			id: self.pcb.id.next(),
		};

		if let Some(next_packet) = self.iter.peek() {
			if !next_packet.filled_payload().is_empty() {
				header.flags |= Flags::MORE_DATA;
			}
		}

		header.write_to(packet.header_mut());

		return Some(packet.initialized());
	}
}
