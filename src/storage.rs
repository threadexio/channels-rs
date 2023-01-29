use core::cmp;
use std::io;

pub struct Buffer {
	inner: Vec<u8>,
	cursor: usize,
}

#[allow(dead_code)]
impl Buffer {
	pub fn new(capacity: usize) -> Self {
		Self { cursor: 0, inner: vec![0u8; capacity] }
	}

	/// Return the maximum number of elements the buffer can hold.
	pub fn capacity(&self) -> usize {
		self.inner.len()
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

	/// Return a inner of the entire buffer.
	pub fn buffer(&self) -> &[u8] {
		&self.inner
	}

	/// Return a mutable inner of the entire buffer.
	pub fn buffer_mut(&mut self) -> &mut [u8] {
		&mut self.inner
	}

	/// Return a inner of all the elements after the cursor.
	pub fn after(&self) -> &[u8] {
		&self.inner[self.cursor..]
	}

	/// Return a mutable inner of all the elements after the cursor.
	pub fn after_mut(&mut self) -> &mut [u8] {
		&mut self.inner[self.cursor..]
	}

	// Return a inner of the elements before the cursor.
	pub fn before(&self) -> &[u8] {
		&self.inner[..self.cursor]
	}

	// Return a mutable inner of the elements before the cursor.
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

pub trait ReadExt: io::Read {
	/// This method repeatedly calls `read()` until:
	///   - `read()` returns an error, except `io::ErrorKind::Interrupted`
	///   - `read()` returns `0`
	///   - `buf` has been filled up to `len`
	fn fill_buf_to(
		&mut self,
		buf: &mut Buffer,
		len: usize,
	) -> io::Result<usize> {
		let start = buf.len();
		let end = cmp::min(len, buf.capacity());

		while buf.len() < end {
			let remaining = end - buf.len();

			use io::ErrorKind;
			match self.read(&mut buf.after_mut()[..remaining]) {
				Ok(0) => {
					return Err(io::Error::new(
						ErrorKind::UnexpectedEof,
						"unexpected eof",
					))
				},
				Ok(v) => {
					buf.seek_forward(v);
				},
				Err(e) if e.kind() == ErrorKind::Interrupted => {},
				Err(e) => return Err(e),
			}
		}

		Ok(buf.len() - start)
	}
}

impl<T: io::Read> ReadExt for T {}
