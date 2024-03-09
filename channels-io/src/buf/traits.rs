use super::{
	chain::{self, Chain},
	limit::{self, Limit},
	take::{self, Take},
};

/// A type that holds a contiguous slice of bytes.
pub trait AsBytes {
	fn as_bytes(&self) -> &[u8];
}

/// A buffer with [`Cursor`] functionality.
///
/// This trait does not guaranteed that the internal representation of the buffer
/// is a contiguous region of memory.
pub trait Buf {
	/// Get the remaining part of the buffer.
	///
	/// A call to [`chunk`] may return a slice shorter than what [`remaining`]
	/// says. This allows for non-contiguous representation of the buffer.
	fn chunk(&self) -> &[u8];

	/// Get the number of bytes remaining in this buffer.
	fn remaining(&self) -> usize;

	/// Advance the start of the buffer by `cnt` bytes.
	///
	/// `cnt` must be in the range `[0, self.remaining()]`.
	fn advance(&mut self, cnt: usize);

	/// Check whether the buffer has any remaining bytes left.
	///
	/// Equivalent to: `self.remaining() != 0`.
	fn has_remaining(&self) -> bool {
		self.remaining() != 0
	}

	/// Create a [`Chain`] adapter between `self` and `other`.
	fn chain<B>(self, other: B) -> Chain<Self, B>
	where
		Self: Sized,
		B: Buf,
	{
		chain::chain(self, other)
	}

	/// Create a [`Take`] adapter that will have at most `limit` bytes.
	fn take(self, limit: usize) -> Take<Self>
	where
		Self: Sized,
	{
		take::take(self, limit)
	}

	/// Create an adapter that takes `self` by mutable reference.
	fn by_ref(&mut self) -> &mut Self
	where
		Self: Sized,
	{
		self
	}
}

/// A marker trait that describes the behavior of [`Buf::chunk`].
///
/// # Safety
///
/// If this trait is implemented, then the slice returned by [`Buf::chunk`] MUST
/// be of length [`Buf::remaining`].
pub unsafe trait Contiguous: Buf {}

/// A non-contiguous buffer whose chunks can be iterated over.
pub trait Walkable<'chunk>: Buf {
	type Iter: Iterator<Item = &'chunk [u8]>;

	fn walk_chunks(&'chunk self) -> Self::Iter;
}

/// A type that holds a contiguous slice of mutable bytes.
pub trait AsBytesMut: AsBytes {
	fn as_bytes_mut(&mut self) -> &mut [u8];
}

/// A mutable buffer with [`Cursor`] functionality.
///
/// This trait does not guarantee that the internal representation of the buffer
/// is a contiguous region of memory.
pub trait BufMut {
	/// Get the remaining part of the buffer.
	///
	/// A call to [`chunk_mut`] may return a slice shorter than what [`remaining_mut`]
	/// says. This allows for non-contiguous representation of the buffer.
	fn chunk_mut(&mut self) -> &mut [u8];

	/// Get the number of bytes remaining in this buffer.
	fn remaining_mut(&self) -> usize;

	/// Advance the start of the buffer by `cnt` bytes.
	///
	/// `cnt` must be in the range `[0, self.remaining_mut()]`.
	fn advance_mut(&mut self, cnt: usize);

	/// Check whether the buffer has any remaining bytes left.
	///
	/// Equivalent to: `self.remaining_mut() != 0`.
	fn has_remaining_mut(&self) -> bool {
		self.remaining_mut() != 0
	}

	/// Create a [`Chain`] adapter between `self` and `other`.
	fn chain<B: BufMut>(self, other: B) -> Chain<Self, B>
	where
		Self: Sized,
	{
		chain::chain(self, other)
	}

	/// Create a [`Limit`] adapter.
	fn limit<B: BufMut>(self, limit: usize) -> Limit<Self>
	where
		Self: Sized,
	{
		limit::limit(self, limit)
	}

	/// Create an adapter that takes `self` by mutable reference.
	fn by_ref(&mut self) -> &mut Self
	where
		Self: Sized,
	{
		self
	}
}

/// A marker trait that describes the behavior of [`BufMut::chunk_mut`].
///
/// # Safety
///
/// If this trait is implemented, then the slice returned by [`BufMut::chunk_mut`]
/// MUST be of length [`BufMut::remaining_mut`].
pub unsafe trait ContiguousMut: BufMut {}

/// A non-contiguous mutable buffer whose chunks can be iterated over.
pub trait WalkableMut<'chunk>: BufMut {
	type Iter: Iterator<Item = &'chunk mut [u8]>;

	fn walk_chunks_mut(&'chunk mut self) -> Self::Iter;
}
