use core::cmp;

pub struct Buffer {
	slice: Vec<u8>,
	cursor: usize,
}

impl Buffer {
	pub fn new(capacity: usize) -> Self {
		Self { cursor: 0, slice: vec![0u8; capacity] }
	}

	/// Return the maximum number of elements the buffer can hold.
	pub fn capacity(&self) -> usize {
		self.slice.len()
	}

	/// Return the number of elements in the buffer.
	pub fn len(&self) -> usize {
		self.cursor
	}

	/// Seek to absolute idx.
	pub fn seek(&mut self, idx: usize) -> usize {
		self.cursor = self.get_idx(idx);
		self.cursor
	}

	pub fn seek_forward(&mut self, off: usize) -> usize {
		self.seek(self.len().saturating_add(off))
	}

	pub fn seek_backward(&mut self, off: usize) -> usize {
		self.seek(self.len().saturating_sub(off))
	}

	pub fn clear(&mut self) {
		self.cursor = 0;
	}

	/// Return a slice of the entire buffer.
	pub fn buffer(&self) -> &[u8] {
		&self.slice
	}

	/// Return a mutable slice of the entire buffer.
	pub fn buffer_mut(&mut self) -> &mut [u8] {
		&mut self.slice
	}

	/// Return a slice of all the elements after the cursor.
	pub fn after(&self) -> &[u8] {
		&self.slice[self.cursor..]
	}

	/// Return a mutable slice of all the elements after the cursor.
	pub fn after_mut(&mut self) -> &mut [u8] {
		&mut self.slice[self.cursor..]
	}

	// Return a slice of the elements before the cursor.
	pub fn before(&self) -> &[u8] {
		&self.slice[..self.cursor]
	}

	// Return a mutable slice of the elements before the cursor.
	pub fn before_mut(&mut self) -> &mut [u8] {
		&mut self.slice[..self.cursor]
	}

	fn get_idx(&self, idx: usize) -> usize {
		cmp::min(idx, self.capacity())
	}
}
