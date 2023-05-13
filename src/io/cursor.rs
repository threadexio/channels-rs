use core::cmp;
use core::fmt;
use std::io;

pub trait BytesRef: AsRef<[u8]> {}
impl<T> BytesRef for T where T: AsRef<[u8]> {}

pub trait BytesMut: BytesRef + AsMut<[u8]> {}
impl<T> BytesMut for T where T: BytesRef + AsMut<[u8]> {}

#[derive(Clone, PartialEq, Eq)]
pub struct Cursor<T> {
	inner: T,
	pos: usize,
}

impl<T> Cursor<T> {
	pub const fn new(inner: T) -> Self {
		Self { inner, pos: 0 }
	}

	/// Get the amount of bytes currently present in the buffer.
	pub const fn len(&self) -> usize {
		self.pos
	}

	/// Get the position of the cursor.
	pub const fn pos(&self) -> usize {
		self.pos
	}
}

impl<T> Cursor<T>
where
	T: BytesRef,
{
	/// Get the entire buffer as a slice.
	pub fn as_slice(&self) -> &[u8] {
		self.inner.as_ref()
	}

	/// Get the limit of bytes the buffer can hold.
	pub fn capacity(&self) -> usize {
		self.as_slice().len()
	}

	/// Set the position of the cursor. This is a low-level
	/// operation that maintains no invariants. Its safe wrappers
	/// (`seek*()`) should be used instead.
	///
	/// # Safety
	///
	/// `new_pos` must be a valid position in the buffer:
	///
	/// - `new_pos` <= `self.capacity()`
	pub unsafe fn set_pos(&mut self, new_pos: usize) {
		debug_assert!(new_pos <= self.capacity());
		self.pos = new_pos;
	}

	/// Clear the buffer of any remaining bytes.
	pub fn clear(&mut self) {
		unsafe {
			self.set_pos(0);
		}
	}

	/// Seek to a specified index.
	///
	/// # Safety
	///
	/// This method will panic if `idx` is
	/// out of bounds.
	pub fn seek(&mut self, pos: usize) {
		if pos > self.capacity() {
			panic!("index out of bounds")
		}

		unsafe {
			self.set_pos(pos);
		}
	}

	/// Seek `off` bytes forwards.
	///
	/// # Safety
	///
	/// See [`Self::seek`].
	pub fn seek_forward(&mut self, off: usize) {
		self.seek(self.pos() + off);
	}

	/// Seek `off` bytes backwards.
	///
	/// # Safety
	///
	/// See [`Self::seek`].
	pub fn seek_backward(&mut self, off: usize) {
		self.seek(self.pos() - off);
	}

	/// Get everything _after_ the cursor. Equivalent to: `&buf[buf.pos()..]`
	pub fn after(&self) -> &[u8] {
		let start = self.pos();
		&self.as_slice()[start..]
	}

	/// Get everything before_ the cursor. Equivalent to: `&buf[..buf.pos()]`
	pub fn before(&self) -> &[u8] {
		let end = self.pos();
		&self.as_slice()[..end]
	}
}

impl<T> Cursor<T>
where
	T: BytesMut,
{
	/// Get the entire buffer as a slice.
	pub fn as_mut_slice(&mut self) -> &mut [u8] {
		self.inner.as_mut()
	}

	/// Get everything _after_ the cursor. Equivalent to: `&mut buf[buf.pos()..]`
	pub fn after_mut(&mut self) -> &mut [u8] {
		let start = self.pos();
		&mut self.as_mut_slice()[start..]
	}

	/// Get everything before_ the cursor. Equivalent to: `&mut buf[..buf.pos()]`
	pub fn before_mut(&mut self) -> &mut [u8] {
		let end = self.pos();
		&mut self.as_mut_slice()[..end]
	}
}

impl<T> io::Read for Cursor<T>
where
	T: BytesRef,
{
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let amt = copy_from_slice_min_len(self.after(), buf);
		self.seek_forward(amt);
		Ok(amt)
	}
}

impl<T> io::Write for Cursor<T>
where
	T: BytesMut,
{
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		let amt = copy_from_slice_min_len(buf, self.after_mut());

		// DEBUG
		eprintln!(
			"wrote {:?} extra bytes. total: {:?}",
			amt,
			self.len()
		);

		self.seek_forward(amt);
		Ok(amt)
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}

fn copy_from_slice_min_len(src: &[u8], dst: &mut [u8]) -> usize {
	let min_len = cmp::min(src.len(), dst.len());
	dst[..min_len].copy_from_slice(&src[..min_len]);
	min_len
}

impl<T> fmt::Debug for Cursor<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Cursor")
			.field("inner", &"[..]")
			.field("pos", &self.pos)
			.finish()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn init_cursor() -> Cursor<Vec<u8>> {
		Cursor::new(vec![0u8; 8])
	}

	#[test]
	fn test_seek() {
		let mut cursor = init_cursor();
		assert_eq!(cursor.pos(), 0);

		cursor.seek(8);
		assert_eq!(cursor.pos(), 8);

		cursor.seek(5);
		assert_eq!(cursor.pos(), 5);

		cursor.seek_backward(3);
		assert_eq!(cursor.pos(), 2);

		cursor.seek_forward(2);
		assert_eq!(cursor.pos(), 4);

		cursor.seek(0);
		assert_eq!(cursor.pos(), 0);
	}

	#[test]
	#[should_panic]
	fn test_seek_out_of_bounds() {
		let mut cursor = init_cursor();
		cursor.seek(9);
	}

	#[test]
	fn test_range_accessors() {
		let mut cursor = init_cursor();
		cursor.seek(5);

		assert_eq!(cursor.len(), 5);

		assert_eq!(cursor.before(), &[0, 0, 0, 0, 0]);
		assert_eq!(cursor.before_mut(), &[0, 0, 0, 0, 0]);

		assert_eq!(cursor.after(), &[0, 0, 0]);
		assert_eq!(cursor.after_mut(), &[0, 0, 0]);
	}

	#[test]
	fn test_read() {
		use std::io::Read;

		let mut cursor = Cursor::new((0..=7).collect::<Vec<u8>>());
		let mut buf = vec![0u8; 10];

		let amt = cursor.read(&mut buf[..4]).unwrap();
		assert_eq!(amt, 4);
		assert_eq!(cursor.pos(), 4);
		assert_eq!(&buf, &[0, 1, 2, 3, 0, 0, 0, 0, 0, 0]);

		let amt = cursor.read(&mut buf[4..]).unwrap();
		assert_eq!(amt, 4);
		assert_eq!(cursor.pos(), 8);
		assert_eq!(&buf, &[0, 1, 2, 3, 4, 5, 6, 7, 0, 0]);
	}

	#[test]
	fn test_write() {
		use std::io::Write;

		let mut cursor = Cursor::new(vec![0_u8; 8]);

		let amt = cursor.write(&[0, 1, 2, 3]).unwrap();
		assert_eq!(amt, 4);
		assert_eq!(cursor.pos(), 4);
		assert_eq!(cursor.as_slice(), &[0, 1, 2, 3, 0, 0, 0, 0]);

		let amt = cursor.write(&[4, 5, 6]).unwrap();
		assert_eq!(amt, 3);
		assert_eq!(cursor.pos(), 7);
		assert_eq!(cursor.as_slice(), &[0, 1, 2, 3, 4, 5, 6, 0]);

		let amt = cursor.write(&[7, 8, 9]).unwrap();
		assert_eq!(amt, 1);
		assert_eq!(cursor.pos(), 8);
		assert_eq!(cursor.as_slice(), &[0, 1, 2, 3, 4, 5, 6, 7]);
	}
}
