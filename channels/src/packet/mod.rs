pub mod consts {
	pub const MAX_PACKET_SIZE: usize = 0xffff;

	pub const HEADER_SIZE: usize =
		super::header::private::HEADER_SIZE;
	pub const MAX_PAYLOAD_SIZE: usize = MAX_PACKET_SIZE - HEADER_SIZE;
}

pub mod header;
pub mod list;
pub mod types;

use types::Id;

#[derive(Debug, Default)]
pub struct Pcb {
	pub id: Id,
}

impl Pcb {
	/// Update this pcb to be ready for the next packet.
	pub fn next(&mut self) {
		self.id = self.id.next();
	}
}
