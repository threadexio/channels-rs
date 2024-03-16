//! Add [`Buf`]/[`BufMut`] functionality to types that do not support it.

use core::iter::{once, Once};

use super::{
	AsBytes, AsBytesMut, Buf, BufMut, Contiguous, ContiguousMut,
	Walkable, WalkableMut,
};

/// Add [`Buf`]/[`BufMut`] functionality to types that do not support it.
#[derive(Debug)]
pub struct Cursor<T> {
	buf: T,
	pos: usize,
}

impl<T> Cursor<T> {
	/// Create a new [`Cursor`] from `buf`.
	pub fn new(buf: T) -> Self {
		Self { buf, pos: 0 }
	}

	/// Get a reference to the buffer.
	pub fn get(&self) -> &T {
		&self.buf
	}

	/// Get a mutable reference to the buffer.
	pub fn get_mut(&mut self) -> &mut T {
		&mut self.buf
	}

	/// Destruct the [`Cursor`] and get back the buffer.
	pub fn into_inner(self) -> T {
		self.buf
	}

	/// Get the cursor position inside the buffer.
	pub fn get_pos(&self) -> usize {
		self.pos
	}

	/// Set the cursor position inside the buffer.
	///
	/// Care must be taken to avoid making the cursor position go out-of-bounds.
	///
	/// # Safety
	///
	/// `pos` must not be greater than the length of the underlying buffer.
	pub unsafe fn set_pos(&mut self, pos: usize) {
		self.pos = pos;
	}
}

impl<T> Cursor<T> {
	#[inline]
	fn _remaining(&self, len: usize) -> usize {
		usize::saturating_sub(len, self.pos)
	}

	#[inline]
	fn _advance(&mut self, cnt: usize, remaining: usize) {
		assert!(
			cnt <= remaining,
			"tried to advance past end of cursor"
		);
		self.pos += cnt;
	}
}

impl<T> Buf for Cursor<T>
where
	T: AsBytes,
{
	fn chunk(&self) -> &[u8] {
		&self.buf.as_bytes()[self.pos..]
	}

	fn remaining(&self) -> usize {
		self._remaining(self.buf.as_bytes().len())
	}

	fn advance(&mut self, cnt: usize) {
		self._advance(cnt, self.remaining());
	}
}

unsafe impl<T> Contiguous for Cursor<T> where T: AsBytes {}

impl<'a, T> Walkable<'a> for Cursor<T>
where
	T: AsBytes,
{
	type Iter = Once<&'a [u8]>;

	fn walk_chunks(&'a self) -> Self::Iter {
		once(self.chunk())
	}
}

impl<T> BufMut for Cursor<T>
where
	T: AsBytesMut,
{
	fn chunk_mut(&mut self) -> &mut [u8] {
		&mut self.buf.as_bytes_mut()[self.pos..]
	}

	fn remaining_mut(&self) -> usize {
		self._remaining(self.buf.as_bytes().len())
	}

	fn advance_mut(&mut self, cnt: usize) {
		self._advance(cnt, self.remaining_mut());
	}
}

unsafe impl<T> ContiguousMut for Cursor<T> where T: AsBytesMut {}

impl<'a, T> WalkableMut<'a> for Cursor<T>
where
	T: AsBytesMut,
{
	type Iter = Once<&'a mut [u8]>;

	fn walk_chunks_mut(&'a mut self) -> Self::Iter {
		once(self.chunk_mut())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	mod buf_impl {
		use super::{Buf, Cursor, Walkable};

		#[test]
		fn basic() {
			let mut cursor = Cursor::new([0u8, 1, 2, 3, 4, 5, 6, 7]);

			assert_eq!(cursor.remaining(), 8);
			assert_eq!(cursor.chunk(), [0, 1, 2, 3, 4, 5, 6, 7]);
			assert!(cursor
				.walk_chunks()
				.eq([[0, 1, 2, 3, 4, 5, 6, 7].as_slice()]));

			cursor.advance(4);
			assert_eq!(cursor.remaining(), 4);
			assert_eq!(cursor.chunk(), [4, 5, 6, 7]);
			assert!(cursor
				.walk_chunks()
				.eq([[4, 5, 6, 7].as_slice()]));
		}

		#[test]
		fn empty() {
			let cursor = Cursor::new([]);

			assert_eq!(cursor.remaining(), 0);
			assert_eq!(cursor.chunk(), []);
			assert!(cursor.walk_chunks().eq([[].as_slice()]));
		}

		#[test]
		fn advance_max() {
			let mut cursor = Cursor::new([0u8, 1, 2, 3, 4, 5, 6, 7]);

			cursor.advance(8);
			assert_eq!(cursor.remaining(), 0);
			assert_eq!(cursor.chunk(), []);
			assert!(cursor.walk_chunks().eq([[].as_slice()]));
		}

		#[test]
		#[should_panic(
			expected = "tried to advance past end of cursor"
		)]
		fn advance_out_of_bounds() {
			let mut cursor = Cursor::new([0u8, 1, 2, 3, 4, 5, 6, 7]);
			cursor.advance(9);
		}
	}

	mod bufmut_impl {
		use super::{BufMut, Cursor, WalkableMut};

		#[test]
		fn basic() {
			let mut cursor = Cursor::new([0u8, 1, 2, 3, 4, 5, 6, 7]);

			assert_eq!(cursor.remaining_mut(), 8);
			assert_eq!(cursor.chunk_mut(), [0, 1, 2, 3, 4, 5, 6, 7]);
			assert!(cursor
				.walk_chunks_mut()
				.eq([[0, 1, 2, 3, 4, 5, 6, 7].as_slice()]));

			cursor.advance_mut(4);
			assert_eq!(cursor.remaining_mut(), 4);
			assert_eq!(cursor.chunk_mut(), [4, 5, 6, 7]);
			assert!(cursor
				.walk_chunks_mut()
				.eq([[4, 5, 6, 7].as_slice()]));
		}

		#[test]
		fn empty() {
			let mut cursor = Cursor::new([]);

			assert_eq!(cursor.remaining_mut(), 0);
			assert_eq!(cursor.chunk_mut(), []);
			assert!(cursor.walk_chunks_mut().eq([[].as_slice()]));
		}

		#[test]
		fn advance_mut_max() {
			let mut cursor = Cursor::new([0u8, 1, 2, 3, 4, 5, 6, 7]);

			cursor.advance_mut(8);
			assert_eq!(cursor.remaining_mut(), 0);
			assert_eq!(cursor.chunk_mut(), []);
			assert!(cursor.walk_chunks_mut().eq([[].as_slice()]));
		}

		#[test]
		#[should_panic(
			expected = "tried to advance past end of cursor"
		)]
		fn advance_mut_out_of_bounds() {
			let mut cursor = Cursor::new([0u8, 1, 2, 3, 4, 5, 6, 7]);
			cursor.advance_mut(9);
		}
	}

	#[test]
	fn get_pos() {
		let mut cursor = Cursor::new([0u8, 1, 2, 3, 4, 5, 6, 7]);

		assert_eq!(cursor.get_pos(), 0);

		cursor.advance(4);
		assert_eq!(cursor.get_pos(), 4);

		cursor.advance(4);
		assert_eq!(cursor.get_pos(), 8);
	}
}
