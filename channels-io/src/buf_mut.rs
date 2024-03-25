use core::iter::{once, Once};

use super::util::copy_min_len;
use super::{chain, limit, Chain, Limit};

/// A mutable buffer.
///
/// This trait does not guarantee that the internal representation of the buffer
/// is a contiguous region of memory.
pub trait BufMut {
	/// Get the remaining part of the buffer.
	///
	/// A call to [`BufMut::chunk_mut()`] may return a slice shorter than what
	/// [`BufMut::remaining_mut()`] returns. This allows for non-contiguous
	/// representation of the buffer.
	fn chunk_mut(&mut self) -> &mut [u8];

	/// Get the number of bytes remaining in this buffer.
	fn remaining_mut(&self) -> usize;

	/// Advance the start of the buffer by `cnt` bytes.
	///
	/// `cnt` must be in the range `[0, self.remaining_mut()]`.
	///
	/// # Panics
	///
	/// May panic if `cnt` is not in that range.
	fn advance_mut(&mut self, cnt: usize);

	/// Check whether the buffer has any remaining bytes left.
	///
	/// Equivalent to: `self.remaining_mut() != 0`.
	fn has_remaining_mut(&self) -> bool {
		self.remaining_mut() != 0
	}

	/// Copy bytes from `slice` into this buffer.
	///
	/// This method will copy bytes from `slice` into the buffer until all bytes
	/// from `slice` have been copied over or the buffer has no more space.
	/// Returns the number of bytes copied into the buffer.
	fn copy_from_slice(&mut self, slice: &[u8]) -> usize {
		let mut i = 0;

		while self.has_remaining_mut() && i < slice.len() {
			let n = copy_min_len(&slice[i..], self.chunk_mut());
			i += n;
			self.advance_mut(n);
		}

		i
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

macro_rules! forward_bufmut_impl {
	() => {
		fn chunk_mut(&mut self) -> &mut [u8] {
			(**self).chunk_mut()
		}

		fn remaining_mut(&self) -> usize {
			(**self).remaining_mut()
		}

		fn advance_mut(&mut self, cnt: usize) {
			(**self).advance_mut(cnt)
		}

		fn has_remaining_mut(&self) -> bool {
			(**self).has_remaining_mut()
		}

		fn copy_from_slice(&mut self, slice: &[u8]) -> usize {
			(**self).copy_from_slice(slice)
		}
	};
}
pub(crate) use forward_bufmut_impl;

/// A non-contiguous mutable buffer whose chunks can be iterated over.
pub trait WalkableMut: BufMut {
	/// Chunk iterator type.
	type Iter<'a>: Iterator<Item = &'a mut [u8]>
	where
		Self: 'a;

	/// Walk each chunk of the buffer in order.
	///
	/// This function returns an [`Iterator`] that will iterate over all chunks
	/// of the buffer in order.
	fn walk_chunks_mut(&mut self) -> Self::Iter<'_>;
}

#[rustfmt::skip]
macro_rules! forward_walkable_mut_impl {
	($to:ty) => {
		type Iter<'a> = <$to>::Iter<'a>
		where
			Self: 'a;

		fn walk_chunks_mut(&mut self) -> Self::Iter<'_> {
			(**self).walk_chunks_mut()
		}
	};
}
pub(crate) use forward_walkable_mut_impl;

/// A marker trait that describes the behavior of [`BufMut::chunk_mut()`].
///
/// # Safety
///
/// If this trait is implemented, then the slice returned by [`BufMut::chunk_mut()`]
/// MUST be of length [`BufMut::remaining_mut()`].
pub unsafe trait ContiguousMut: BufMut + WalkableMut {}

// ========================================================

impl<B: BufMut> BufMut for &mut B {
	forward_bufmut_impl!();
}

impl<B: WalkableMut> WalkableMut for &mut B {
	forward_walkable_mut_impl!(B);
}

unsafe impl<B: ContiguousMut> ContiguousMut for &mut B {}

impl BufMut for &mut [u8] {
	fn chunk_mut(&mut self) -> &mut [u8] {
		self
	}

	fn remaining_mut(&self) -> usize {
		self.len()
	}

	fn advance_mut(&mut self, cnt: usize) {
		assert!(
			cnt <= self.remaining_mut(),
			"tried to advance past end of slice"
		);
		let tmp = core::mem::take(self);
		*self = &mut tmp[cnt..];
	}
}

impl WalkableMut for &mut [u8] {
	type Iter<'a> = Once<&'a mut [u8]>
	where
		Self: 'a;

	fn walk_chunks_mut(&mut self) -> Self::Iter<'_> {
		once(*self)
	}
}

unsafe impl ContiguousMut for &mut [u8] {}

#[cfg(feature = "alloc")]
mod alloc_impls {
	use super::{forward_bufmut_impl, forward_walkable_mut_impl};
	use super::{BufMut, ContiguousMut, WalkableMut};

	#[allow(unused_imports)]
	use alloc::boxed::Box;

	impl<B: BufMut> BufMut for Box<B> {
		forward_bufmut_impl!();
	}

	impl<B: WalkableMut> WalkableMut for Box<B> {
		forward_walkable_mut_impl!(B);
	}

	unsafe impl<B: ContiguousMut> ContiguousMut for Box<B> {}
}
