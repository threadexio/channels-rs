pub mod consts {
	use super::*;

	pub const MAX_PACKET_SIZE: usize = 0xffff;
	pub const HEADER_SIZE: usize = Header::SIZE;

	pub const MAX_PAYLOAD_SIZE: usize = MAX_PACKET_SIZE - HEADER_SIZE;
}

pub mod header;
use header::*;

mod block;
pub use block::Block;

mod linked;
pub use linked::LinkedBlocks;

pub mod packet;

#[derive(Debug, Default)]
pub struct Pcb {
	pub id: Id,
}
