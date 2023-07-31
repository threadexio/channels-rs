use super::block::Block;
use super::consts::*;
use super::header::*;
use super::Pcb;

#[derive(Debug)]
pub struct LinkedBlocks {
	pub blocks: Vec<Block>,
}

impl LinkedBlocks {
	pub fn new() -> Self {
		Self { blocks: Vec::new() }
	}

	pub fn with_payload_capacity(capacity: usize) -> Self {
		let mut list = Self { blocks: Vec::new() };

		list.ensure_capacity(capacity);

		list
	}

	/// Calculate the current capacity of the list.
	pub fn total_capacity(&self) -> usize {
		self.blocks.iter().map(|block| block.capacity()).sum()
	}

	/// Ensure the list has enough capacity to hold `capacity` bytes
	/// reallocating if necessary.
	pub fn ensure_capacity(&mut self, capacity: usize) {
		let cur_capacity = self.total_capacity();
		if cur_capacity >= capacity {
			return;
		}

		let delta_capacity = capacity - cur_capacity;

		let n_full_blocks = delta_capacity / MAX_PACKET_SIZE;
		let extra_bytes = delta_capacity % MAX_PACKET_SIZE;

		let n_blocks = {
			if extra_bytes != 0 {
				n_full_blocks + 1
			} else {
				n_full_blocks
			}
		};

		self.blocks.reserve(n_blocks);

		for _ in 0..n_full_blocks {
			self.blocks
				.push(Block::with_payload_capacity(MAX_PAYLOAD_SIZE));
		}

		if extra_bytes != 0 {
			self.blocks
				.push(Block::with_payload_capacity(extra_bytes));
		}
	}

	pub fn clear_all(&mut self) {
		self.blocks.iter_mut().for_each(|block| block.clear());
	}

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

			if block.is_payload_full() {
				if let Some(next_block) = block_iter.peek() {
					if !next_block.is_payload_empty() {
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
