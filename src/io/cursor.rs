use core::cmp;
use core::fmt;

use super::{Read, Result, Write};

pub trait BytesRef: AsRef<[u8]> {}
impl<T> BytesRef for T where T: AsRef<[u8]> {}

pub trait BytesMut: BytesRef + AsMut<[u8]> {}
impl<T> BytesMut for T where T: BytesRef + AsMut<[u8]> {}

#[derive(Clone)]
pub struct Cursor<T> {
	inner: T,
	pos: usize,
}

impl<T> Cursor<T> {
	pub fn new(inner: T) -> Self {
		Self { inner, pos: 0 }
	}

	pub fn get(&self) -> &T {
		&self.inner
	}

	pub fn get_mut(&mut self) -> &mut T {
		&mut self.inner
	}

	pub fn into_inner(self) -> T {
		self.inner
	}
}

impl<T> Cursor<T>
where
	T: BytesRef,
{
	pub fn as_slice(&self) -> &[u8] {
		self.inner.as_ref()
	}

	pub fn remaining(&self) -> &[u8] {
		&self.inner.as_ref()[self.pos..]
	}

	pub fn len(&self) -> usize {
		self.as_slice().len()
	}

	pub fn position(&self) -> usize {
		self.pos
	}

	pub fn set_position(&mut self, position: usize) {
		self.pos = position;
	}

	/// Advance the cursor by `n` bytes.
	///
	/// Returns `Some(new_pos)` or `None` if the new position is out
	/// of bounds.
	#[must_use = "unused advance result"]
	pub fn advance(&mut self, n: usize) -> Option<usize> {
		let new_pos = self.pos + n;

		if new_pos <= self.len() {
			self.pos = new_pos;
			Some(new_pos)
		} else {
			None
		}
	}
}

impl<T> Cursor<T>
where
	T: BytesMut,
{
	pub fn as_mut_slice(&mut self) -> &mut [u8] {
		self.inner.as_mut()
	}

	pub fn remaining_mut(&mut self) -> &mut [u8] {
		&mut self.inner.as_mut()[self.pos..]
	}
}

impl<T> fmt::Debug for Cursor<T>
where
	T: BytesRef,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Cursor")
			.field("inner", &self.as_slice())
			.field("pos", &self.pos)
			.finish()
	}
}

impl<T> Read for Cursor<T>
where
	T: BytesRef,
{
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let src = self.remaining();
		let n = copy_from_slice_min_len(src, buf);

		// SAFETY:
		//
		// (1) n <= src.len()          , see `copy_from_slice_min_len`
		// (2) src.len() <= self.len() , as self.pos is always in-bounds
		//
		// (1), (2) <=> n <= src.len() <= self.len()
		//          <=> n <= self.len()
		//
		// So the cursor can always move forward by `n` bytes.
		// This means the following unwrap cannot panic.
		self.advance(n).unwrap();

		Ok(n)
	}
}

impl<T> Write for Cursor<T>
where
	T: BytesMut,
{
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		let dst = self.remaining_mut();
		let n = copy_from_slice_min_len(buf, dst);

		// SAFETY:
		//
		// (1) n <= dst.len()          , see `copy_slice_min_len``
		// (2) dst.len() <= self.len() , as self.pos is always in-bounds
		//
		// (1), (2) <=> n <= dst.len() <= self.len()
		//          <=> n <= self.len()
		//
		// So the cursor can always move forward by `n` bytes.
		// This means the following unwrap cannot panic.
		self.advance(n).unwrap();

		Ok(n)
	}

	fn flush(&mut self) -> Result<()> {
		Ok(())
	}
}

/// Copy the minimum length possible from `src` into `dst`.
///
/// Returns the number of bytes copied. The following holds
/// for the return value:
/// - `n <= src.len()`
/// - `n <= dst.len()`
fn copy_from_slice_min_len(src: &[u8], dst: &mut [u8]) -> usize {
	let min_len = cmp::min(src.len(), dst.len());
	dst[..min_len].copy_from_slice(&src[..min_len]);
	min_len
}

#[cfg(test)]
mod tests {
	use super::*;

	fn init_cursor() -> Cursor<Vec<u8>> {
		Cursor::new(vec![0u8; 8])
	}

	#[test]
	fn test_advance() {
		let mut cursor = init_cursor();
		assert_eq!(cursor.position(), 0);

		assert_eq!(cursor.advance(3), Some(3));
		assert_eq!(cursor.advance(1), Some(4));
		assert_eq!(cursor.advance(10), None);
	}

	#[test]
	#[should_panic]
	fn test_seek_out_of_bounds() {
		let mut cursor = init_cursor();
		cursor.advance(9).unwrap();
	}

	#[test]
	fn test_read() {
		use std::io::Read;

		let mut cursor = Cursor::new((0..=7).collect::<Vec<u8>>());
		let mut buf = vec![0u8; 10];

		let amt = cursor.read(&mut buf[..4]).unwrap();
		assert_eq!(amt, 4);
		assert_eq!(cursor.position(), 4);
		assert_eq!(&buf, &[0, 1, 2, 3, 0, 0, 0, 0, 0, 0]);

		let amt = cursor.read(&mut buf[4..]).unwrap();
		assert_eq!(amt, 4);
		assert_eq!(cursor.position(), 8);
		assert_eq!(&buf, &[0, 1, 2, 3, 4, 5, 6, 7, 0, 0]);
	}

	#[test]
	fn test_write() {
		use std::io::Write;

		let mut cursor = Cursor::new(vec![0_u8; 8]);

		let amt = cursor.write(&[0, 1, 2, 3]).unwrap();
		assert_eq!(amt, 4);
		assert_eq!(cursor.position(), 4);
		assert_eq!(cursor.as_slice(), &[0, 1, 2, 3, 0, 0, 0, 0]);

		let amt = cursor.write(&[4, 5, 6]).unwrap();
		assert_eq!(amt, 3);
		assert_eq!(cursor.position(), 7);
		assert_eq!(cursor.as_slice(), &[0, 1, 2, 3, 4, 5, 6, 0]);

		let amt = cursor.write(&[7, 8, 9]).unwrap();
		assert_eq!(amt, 1);
		assert_eq!(cursor.position(), 8);
		assert_eq!(cursor.as_slice(), &[0, 1, 2, 3, 4, 5, 6, 7]);
	}
}
