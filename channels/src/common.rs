#![allow(unused)]

use core::fmt;

use channels_packet::IdGenerator;

#[derive(Clone)]
pub struct Pcb {
	pub id_gen: IdGenerator,
}

impl Pcb {
	pub const fn new() -> Self {
		Self { id_gen: IdGenerator::new() }
	}
}

/// IO statistic information.
#[derive(Clone)]
pub struct Statistics {
	total_bytes: u64,
	packets: u64,
	ops: u64,
}

impl Statistics {
	pub(crate) const fn new() -> Self {
		Self { total_bytes: 0, packets: 0, ops: 0 }
	}

	#[inline]
	pub(crate) fn add_total_bytes(&mut self, n: u64) {
		self.total_bytes += n;
	}

	#[inline]
	pub(crate) fn inc_packets(&mut self) {
		self.packets += 1;
	}

	#[inline]
	pub(crate) fn inc_ops(&mut self) {
		self.ops += 1;
	}
}

impl Statistics {
	/// Returns the number of bytes transferred through this reader/writer.
	pub fn total_bytes(&self) -> u64 {
		self.total_bytes
	}

	/// Returns the number of packets transferred through this reader/writer.
	pub fn packets(&self) -> u64 {
		self.packets
	}

	/// Returns the total number of `send`/`recv` operations.
	pub fn ops(&self) -> u64 {
		self.ops
	}
}

impl fmt::Debug for Statistics {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Statistics")
			.field("total_bytes", &self.total_bytes)
			.field("packets", &self.packets)
			.field("ops", &self.ops)
			.finish()
	}
}
