use core::mem;

/// An immutable buffer that contains an internal cursor.
pub trait Buf {
	/// Get the number of bytes remaining in the buffer.
	fn remaining(&self) -> usize;

	/// Get the remaining bytes of the buffer as a slice.
	///
	/// This method is allowed to return slices smaller than what [`Buf::remaining()`]
	/// describes. This allows non contiguous representation of the buffer in memory.
	fn chunk(&self) -> &[u8];

	/// Advance the internal cursor of the buffer by `n` bytes.
	///
	/// # Panics
	///
	/// If `n` is a value that would cause the cursor to go out of bounds.
	fn advance(&mut self, n: usize);

	/// Check whether the buffer has any data left in it.
	fn has_remaining(&self) -> bool {
		self.remaining() != 0
	}
}

macro_rules! forward_impl_buf {
	($to:ty) => {
		fn remaining(&self) -> usize {
			<$to>::remaining(self)
		}

		fn chunk(&self) -> &[u8] {
			<$to>::chunk(self)
		}

		fn advance(&mut self, n: usize) {
			<$to>::advance(self, n);
		}

		fn has_remaining(&self) -> bool {
			<$to>::has_remaining(self)
		}
	};
}

impl<T: Buf + ?Sized> Buf for &mut T {
	forward_impl_buf!(T);
}

#[cfg(feature = "alloc")]
impl<T: Buf + ?Sized> Buf for alloc::boxed::Box<T> {
	forward_impl_buf!(T);
}

impl Buf for &[u8] {
	fn remaining(&self) -> usize {
		self.len()
	}

	fn chunk(&self) -> &[u8] {
		self
	}

	fn advance(&mut self, n: usize) {
		*self = &self[n..];
	}
}

/// A mutable and initialized buffer that contains an internal cursor.
pub trait BufMut {
	/// Get the number of bytes this buffer can hold.
	fn remaining_mut(&self) -> usize;

	/// Get the remaining part of the buffer as a mutable slice.
	///
	/// This method is allowed to return slices smaller than what [`BufMut::remaining_mut()`]
	/// describes. This allows non contiguous representation of the buffer in memory.
	fn chunk_mut(&mut self) -> &mut [u8];

	/// Advance the internal cursor of the buffer by `n` bytes.
	///
	/// # Panics
	///
	/// If `n` is a value that would cause the cursor to go out of bounds.
	fn advance_mut(&mut self, n: usize);

	/// Check whether the buffer has any data left in it.
	fn has_remaining_mut(&self) -> bool {
		self.remaining_mut() != 0
	}
}

macro_rules! forward_impl_buf_mut {
	($to:ty) => {
		fn remaining_mut(&self) -> usize {
			<$to>::remaining_mut(self)
		}

		fn chunk_mut(&mut self) -> &mut [u8] {
			<$to>::chunk_mut(self)
		}

		fn advance_mut(&mut self, n: usize) {
			<$to>::advance_mut(self, n)
		}

		fn has_remaining_mut(&self) -> bool {
			<$to>::has_remaining_mut(self)
		}
	};
}

impl<T: BufMut + ?Sized> BufMut for &mut T {
	forward_impl_buf_mut!(T);
}

#[cfg(feature = "alloc")]
impl<T: BufMut + ?Sized> BufMut for alloc::boxed::Box<T> {
	forward_impl_buf_mut!(T);
}

impl BufMut for &mut [u8] {
	fn remaining_mut(&self) -> usize {
		self.len()
	}

	fn chunk_mut(&mut self) -> &mut [u8] {
		self
	}

	fn advance_mut(&mut self, n: usize) {
		let tmp = mem::take(self);
		*self = &mut tmp[n..];
	}
}

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
