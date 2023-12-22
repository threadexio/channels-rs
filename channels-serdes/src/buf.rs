use core::fmt;

use alloc::collections::VecDeque;

const MAX_CHUNK_SIZE: usize =
	channels_packet::PayloadLength::MAX.as_usize();

/// A buffer of packet-sized buffers.
#[derive(Clone)]
pub struct PayloadBuffer {
	chunks: VecDeque<Vec<u8>>,
}

impl fmt::Debug for PayloadBuffer {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		struct DebugChunk<'a>(&'a Vec<u8>);

		impl fmt::Debug for DebugChunk<'_> {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				f.debug_struct("Chunk")
					.field("len", &self.0.len())
					.finish()
			}
		}

		f.debug_list()
			.entries(self.chunks.iter().map(DebugChunk))
			.finish()
	}
}

impl PayloadBuffer {
	/// Create a new empty buffer.
	pub const fn new() -> Self {
		Self { chunks: VecDeque::new() }
	}

	/// Get the number of chunks present in the list.
	pub fn chunk_count(&self) -> usize {
		self.chunks.len()
	}

	/// Get the sum of the number of bytes in every buffer of the list.
	pub fn len(&self) -> usize {
		self.chunks.iter().map(|chunk| chunk.len()).sum()
	}

	/// Check whether any of the buffers holds any data.
	pub fn is_empty(&self) -> bool {
		self.chunks.iter().any(|chunk| chunk.len() > 0)
	}

	/// Reserve `n` bytes total in the buffer allocating more chunks if needed.
	///
	/// It is generally good practice to try to call `reserve` before writing
	/// to the list so as to minimize the number of allocations done.
	pub fn reserve(&mut self, n: usize) {
		if n == 0 {
			return;
		}

		let full_chunks = n / MAX_CHUNK_SIZE;
		let remaining_bytes = n % MAX_CHUNK_SIZE;

		self.chunks
			.resize_with(self.chunks.len() + full_chunks, || {
				Vec::with_capacity(MAX_CHUNK_SIZE)
			});

		if remaining_bytes != 0 {
			self.chunks
				.push_back(Vec::with_capacity(remaining_bytes));
		}
	}

	/// Write a slice `s` to the buffer.
	#[allow(clippy::missing_panics_doc)]
	pub fn put_slice(&mut self, mut s: &[u8]) {
		use core::cmp::min;

		while !s.is_empty() {
			let chunk = match self
				.chunks
				.iter_mut()
				.find(|chunk| chunk.len() < MAX_CHUNK_SIZE)
			{
				Some(chunk) => chunk,
				None => {
					let capacity = min(s.len(), MAX_CHUNK_SIZE);
					let chunk = Vec::with_capacity(capacity);
					self.chunks.push_back(chunk);
					self.chunks.back_mut().unwrap()
				},
			};

			let n = min(s.len(), MAX_CHUNK_SIZE - chunk.len());
			chunk.extend_from_slice(&s[..n]);
			s = &s[n..];
		}
	}

	/// Pop the first chunk of the list.
	pub fn pop_first(&mut self) -> Option<Vec<u8>> {
		self.chunks.pop_front()
	}
}

mod std_impl {
	use super::*;

	use std::io::{Result, Write};

	impl Write for PayloadBuffer {
		fn write(&mut self, buf: &[u8]) -> Result<usize> {
			self.put_slice(buf);
			Ok(buf.len())
		}

		fn flush(&mut self) -> Result<()> {
			Ok(())
		}
	}
}
