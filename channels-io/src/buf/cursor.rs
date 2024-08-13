use crate::buf::{Buf, BufMut};

/// A cursor that adds [`Buf`] and/or [`BufMut`] functionality to types that can
/// be represented by byte slices.
#[derive(Debug, Clone)]
pub struct Cursor<T> {
	inner: T,
	pos: usize,
}

impl<T> Cursor<T> {
	/// Create a new [`Cursor`] which has its position initialized to 0.
	pub fn new(inner: T) -> Self {
		Self { inner, pos: 0 }
	}

	/// Get a reference to the underlying type.
	pub fn get(&self) -> &T {
		&self.inner
	}

	/// Get a mutable reference to the underlying type.
	///
	/// # Safety
	///
	/// Care should be taken to ensure that any resizing of the underlying type
	/// does not silently cause the cursor to go out of bounds.
	pub fn get_mut(&mut self) -> &mut T {
		&mut self.inner
	}

	/// Get the position of the cursor.
	pub fn pos(&self) -> usize {
		self.pos
	}

	/// Set the position of the cursor without doing any bounds checks.
	///
	/// # Safety
	///
	/// `pos` must point inside the buffer.
	pub unsafe fn set_pos_unchecked(&mut self, pos: usize) {
		self.pos = pos;
	}
}

impl<T: AsRef<[u8]>> Cursor<T> {
	/// Set the position of the cursor.
	///
	/// # Panics
	///
	/// If `pos` is out of bounds.
	pub fn set_pos(&mut self, pos: usize) {
		assert!(
			pos <= self.as_slice().len(),
			"pos should point inside the buffer"
		);
		unsafe { self.set_pos_unchecked(pos) }
	}

	fn as_slice(&self) -> &[u8] {
		self.inner.as_ref()
	}
}

impl<T: AsRef<[u8]>> Buf for Cursor<T> {
	fn remaining(&self) -> usize {
		usize::checked_sub(self.as_slice().len(), self.pos)
			.expect("pos should never be greater than the length")
	}

	fn chunk(&self) -> &[u8] {
		&self.as_slice()[self.pos..]
	}

	fn advance(&mut self, n: usize) {
		self.pos = usize::saturating_add(self.pos, n);
	}
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Cursor<T> {
	fn as_slice_mut(&mut self) -> &mut [u8] {
		self.inner.as_mut()
	}
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> BufMut for Cursor<T> {
	fn remaining_mut(&self) -> usize {
		usize::checked_sub(self.as_slice().len(), self.pos)
			.expect("pos should never be greater than the length")
	}

	fn chunk_mut(&mut self) -> &mut [u8] {
		let pos = self.pos;
		&mut self.as_slice_mut()[pos..]
	}

	fn advance_mut(&mut self, n: usize) {
		self.pos = usize::saturating_add(self.pos, n);
	}
}
