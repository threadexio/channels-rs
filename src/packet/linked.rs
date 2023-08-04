use super::block::Block;
use super::consts::*;
use super::types::*;
use super::Header;
use super::Pcb;

#[derive(Debug)]
pub struct LinkedBlocks {
	pub blocks: Vec<Block>,
}

impl LinkedBlocks {
	/// Create a new empty list.
	///
	/// This method does not allocate.
	pub fn new() -> Self {
		Self { blocks: Vec::new() }
	}

	/// Create a new list with `capacity` bytes of total payload capacity.
	pub fn with_total_payload_capacity(capacity: usize) -> Self {
		let mut list = Self::new();
		list.reserve_payload(capacity);
		list
	}
}

impl LinkedBlocks {
	/// Calculate the total capacity of the list.
	pub fn total_packet_capacity(&self) -> usize {
		self.blocks
			.iter()
			.map(|block| {
				let x: usize = block.packet_capacity().into();
				x
			})
			.sum()
	}

	/// Calculate the total payload capacity of the list.
	pub fn total_payload_capacity(&self) -> usize {
		self.blocks
			.iter()
			.map(|block| {
				let x: usize = block.payload_capacity().into();
				x
			})
			.sum()
	}
}

impl LinkedBlocks {
	/// Reserve enough space for the list to be able to hold a total
	/// of `new_capacity` bytes worth of payload.
	pub fn reserve_payload(&mut self, new_capacity: usize) {
		let capacity = self.total_payload_capacity();

		if new_capacity <= capacity {
			return;
		}
		let delta = new_capacity - capacity;
		let n_full_blocks = delta / MAX_PAYLOAD_SIZE;
		let extra_bytes = delta % MAX_PAYLOAD_SIZE;

		let n_blocks = {
			if extra_bytes != 0 {
				n_full_blocks + 1
			} else {
				n_full_blocks
			}
		};

		self.blocks.reserve(n_blocks);

		// allocate the blocks
		for _ in 0..n_full_blocks {
			let block =
				Block::with_payload_capacity(PayloadLength::MAX);
			self.blocks.push(block);
		}

		// allocate the extras
		if extra_bytes != 0 {
			// SAFETY: extra_bytes = delta % MAX_PAYLOAD_SIZE
			//     <=> extra_bytes < MAX_PAYLOAD_SIZE
			let l = PayloadLength::try_from(extra_bytes).unwrap();

			let block = Block::with_payload_capacity(l);
			self.blocks.push(block);
		}
	}
}

impl LinkedBlocks {
	/// Clear the payload buffer of every block in the lists.
	pub fn clear_payload(&mut self) {
		self.blocks
			.iter_mut()
			.for_each(|block| block.clear_payload());
	}
}

impl LinkedBlocks {
	/// Finalize the blocks and prepare them to be sent.
	///
	/// Returns the blocks that need to be sent.
	pub fn finalize(&mut self, pcb: &mut Pcb) -> &[Block] {
		let mut end_block_idx = 0;

		let mut block_iter = self.blocks.iter_mut().peekable();

		while let Some(block) = block_iter.next() {
			let mut header = Header {
				length: block.packet_length(),
				flags: Flags::zero(),
				id: pcb.id,
			};

			pcb.id = pcb.id.next();

			if block.remaining_payload().is_empty() {
				if let Some(next_block) = block_iter.peek() {
					if !next_block.filled_payload().is_empty() {
						header.flags |= Flags::MORE_DATA;
						end_block_idx += 1;
					}
				}
			}

			header.write_to(block.header_mut());

			if !(header.flags & Flags::MORE_DATA) {
				break;
			}
		}

		&self.blocks[..=end_block_idx]
	}
}

use std::io;

impl io::Write for LinkedBlocks {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		if buf.is_empty() {
			return Ok(0);
		}

		let mut n = 0;
		let mut i = 0;

		loop {
			if self.blocks.get(i).is_none() {
				self.blocks.push(Block::new());
			}

			let block = &mut self.blocks[i];

			n += block.write(&buf[n..])?;
			if n == buf.len() {
				break;
			}

			i += 1;
		}

		Ok(n)
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}

impl io::Read for &mut LinkedBlocks {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		if buf.is_empty() {
			return Ok(0);
		}

		let mut n = 0;

		for block in self.blocks.iter_mut() {
			n += block.read(&mut buf[n..])?;

			if n >= buf.len() {
				break;
			}
		}

		Ok(n)
	}
}
