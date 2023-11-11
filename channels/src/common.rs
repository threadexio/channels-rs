#![allow(unused)]

use channels_packet::IdGenerator;

#[derive(Debug, Clone)]
pub struct Pcb {
	pub id_gen: IdGenerator,
}

impl Pcb {
	pub const fn new() -> Self {
		Self { id_gen: IdGenerator::new() }
	}
}

/// IO statistic information.
#[derive(Debug, Clone)]
pub struct Statistics {
	total_bytes: u64,
}

impl Statistics {
	pub(crate) const fn new() -> Self {
		Self { total_bytes: 0 }
	}

	/// Returns the number of bytes transferred through this reader/writer.
	pub fn total_bytes(&self) -> u64 {
		self.total_bytes
	}

	pub(crate) fn add_total_bytes(&mut self, n: u64) {
		self.total_bytes = u64::saturating_add(self.total_bytes, n)
	}
}
