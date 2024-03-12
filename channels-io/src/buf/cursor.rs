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

impl<T> Buf for Cursor<T>
where
	T: AsBytes,
{
	fn chunk(&self) -> &[u8] {
		&self.buf.as_bytes()[self.pos..]
	}

	fn remaining(&self) -> usize {
		usize::saturating_sub(self.buf.as_bytes().len(), self.pos)
	}

	fn advance(&mut self, cnt: usize) {
		assert!(
			cnt <= self.remaining(),
			"tried to advance past end of cursor"
		);
		self.pos += cnt;
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
		usize::saturating_sub(self.buf.as_bytes().len(), self.pos)
	}

	fn advance_mut(&mut self, cnt: usize) {
		assert!(
			cnt <= self.remaining(),
			"tried to advance past end of cursor"
		);
		self.pos += cnt;
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
