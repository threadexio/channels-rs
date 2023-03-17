use core::cmp;

pub struct Buffer {
	inner: Vec<u8>,
	cursor: usize,
}

#[allow(dead_code)]
impl Buffer {
	pub fn new(capacity: usize) -> Self {
		Self { cursor: 0, inner: vec![0u8; capacity] }
	}

	pub fn capacity(&self) -> usize {
		self.inner.len()
	}

	pub fn len(&self) -> usize {
		self.cursor
	}

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

	pub fn buffer(&self) -> &[u8] {
		&self.inner
	}

	pub fn buffer_mut(&mut self) -> &mut [u8] {
		&mut self.inner
	}

	pub fn after(&self) -> &[u8] {
		&self.inner[self.cursor..]
	}

	pub fn after_mut(&mut self) -> &mut [u8] {
		&mut self.inner[self.cursor..]
	}

	pub fn before(&self) -> &[u8] {
		&self.inner[..self.cursor]
	}

	pub fn before_mut(&mut self) -> &mut [u8] {
		&mut self.inner[..self.cursor]
	}

	fn get_idx(&self, idx: usize) -> usize {
		cmp::min(idx, self.capacity())
	}
}

impl std::ops::Deref for Buffer {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl std::ops::DerefMut for Buffer {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}
