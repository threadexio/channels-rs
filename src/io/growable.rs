use core::ops::{Deref, DerefMut};

use super::{Cursor, Result, Write};

#[derive(Debug)]
pub struct GrowableBuffer(Cursor<Vec<u8>>);

impl GrowableBuffer {
	pub fn new() -> Self {
		Self(Cursor::new(Vec::new()))
	}

	pub fn with_capacity(capacity: usize) -> Self {
		Self(Cursor::new(vec![0u8; capacity]))
	}

	pub fn grow(&mut self, extra: usize) {
		let new_len = self.0.len() + extra;
		self.0.get_mut().resize(new_len, 0);
	}

	pub fn into_inner(self) -> Vec<u8> {
		self.0.into_inner()
	}
}

impl Deref for GrowableBuffer {
	type Target = Cursor<Vec<u8>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for GrowableBuffer {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Write for GrowableBuffer {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		let rem = self.0.remaining().len();
		if buf.len() > rem {
			let extra = buf.len() - rem;
			self.grow(extra);
		}

		self.0.write(buf)
	}

	fn flush(&mut self) -> Result<()> {
		Ok(())
	}
}

impl AsRef<[u8]> for GrowableBuffer {
	fn as_ref(&self) -> &[u8] {
		self.as_slice()
	}
}

impl AsMut<[u8]> for GrowableBuffer {
	fn as_mut(&mut self) -> &mut [u8] {
		self.as_mut_slice()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_growable_buffer() {
		let mut b = GrowableBuffer::new();

		assert_eq!(b.write(&[1, 2, 3, 4]).unwrap(), 4);
		assert_eq!(b.as_slice(), &[1, 2, 3, 4]);

		assert_eq!(
			b.write(&[5, 6, 7, 8, 9, 10, 11, 12, 13]).unwrap(),
			9
		);

		let b = b.into_inner();
		assert_eq!(b.len(), 13);
		assert_eq!(
			b.as_slice(),
			&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]
		);
	}
}
