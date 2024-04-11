//! Traits and utilities to work with mutable buffers.

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

	/// Create a [`Writer`] adapter that implements [`std::io::Write`].
	#[cfg(feature = "std")]
	fn writer(self) -> Writer<Self>
	where
		Self: Sized,
	{
		Writer::new(self)
	}
}

macro_rules! forward_bufmut_impl {
	($to:ty) => {
		fn chunk_mut(&mut self) -> &mut [u8] {
			<$to>::chunk_mut(self)
		}

		fn remaining_mut(&self) -> usize {
			<$to>::remaining_mut(self)
		}

		fn advance_mut(&mut self, cnt: usize) {
			<$to>::advance_mut(self, cnt)
		}

		fn has_remaining_mut(&self) -> bool {
			<$to>::has_remaining_mut(self)
		}

		fn copy_from_slice(&mut self, slice: &[u8]) -> usize {
			<$to>::copy_from_slice(self, slice)
		}
	};
}

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
			<$to>::walk_chunks_mut(self)
		}
	};
}

/// A marker trait that describes the behavior of [`BufMut::chunk_mut()`].
///
/// # Safety
///
/// If this trait is implemented, then the slice returned by [`BufMut::chunk_mut()`]
/// MUST be of length [`BufMut::remaining_mut()`].
pub unsafe trait ContiguousMut: BufMut + WalkableMut {}

// ========================================================

impl<B: BufMut> BufMut for &mut B {
	forward_bufmut_impl!(B);
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
	use super::{BufMut, ContiguousMut, WalkableMut};

	#[allow(unused_imports)]
	use alloc::boxed::Box;

	impl<B: BufMut> BufMut for Box<B> {
		forward_bufmut_impl!(B);
	}

	impl<B: WalkableMut> WalkableMut for Box<B> {
		forward_walkable_mut_impl!(B);
	}

	unsafe impl<B: ContiguousMut> ContiguousMut for Box<B> {}
}

#[cfg(feature = "std")]
mod std_impls {
	use super::BufMut;

	use std::io;

	/// An adapter for [`BufMut`] that implements [`std::io::Write`].
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Writer<B>
	where
		B: BufMut,
	{
		buf: B,
	}

	impl<B: BufMut> Writer<B> {
		pub(crate) fn new(buf: B) -> Self {
			Self { buf }
		}

		/// Get a reference to the buffer.
		pub fn get(&self) -> &B {
			&self.buf
		}

		/// Get a mutable reference to the buffer.
		pub fn get_mut(&mut self) -> &mut B {
			&mut self.buf
		}

		/// Destruct the [`Writer`] and get back the buffer.
		pub fn into_inner(self) -> B {
			self.buf
		}
	}

	impl<B: BufMut> io::Write for Writer<B> {
		fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
			let n = self.buf.copy_from_slice(buf);
			Ok(n)
		}

		fn flush(&mut self) -> io::Result<()> {
			Ok(())
		}
	}
}

#[cfg(feature = "std")]
pub use self::std_impls::Writer;
