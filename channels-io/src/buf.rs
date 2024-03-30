//! Traits and utilities to work with immutable buffers.

use core::iter::{once, Once};

use super::util::copy_min_len;
use super::{chain, take, Chain, Take};

/// An immutable buffer.
///
/// This trait does not guarantee that the internal representation of the buffer
/// is a contiguous region of memory.
pub trait Buf {
	/// Get the remaining part of the buffer.
	///
	/// A call to [`Buf::chunk()`] may return a slice shorter than what
	/// [`Buf::remaining()`] returns. This allows for non-contiguous
	/// representation of the buffer.
	fn chunk(&self) -> &[u8];

	/// Get the number of bytes remaining in this buffer.
	fn remaining(&self) -> usize;

	/// Advance the start of the buffer by `cnt` bytes.
	///
	/// `cnt` must be in the range `[0, self.remaining()]`.
	///
	/// # Panics
	///
	/// May panic if `cnt` is not in that range.
	fn advance(&mut self, cnt: usize);

	/// Check whether the buffer has any remaining bytes left.
	///
	/// Equivalent to: `self.remaining() != 0`.
	fn has_remaining(&self) -> bool {
		self.remaining() != 0
	}

	/// Copy bytes from this buffer into `slice`.
	///
	/// This method will copy as many bytes as needed to fill `slice` from the
	/// buffer and advance it. Returns the number of bytes that were copied to
	/// `slice`.
	fn copy_to_slice(&mut self, slice: &mut [u8]) -> usize {
		let mut i = 0;

		while self.has_remaining() && i < slice.len() {
			let n = copy_min_len(self.chunk(), &mut slice[i..]);
			i += n;
			self.advance(n);
		}

		i
	}

	/// Copy this buffer to a new buffer that is [`Contiguous`].
	#[cfg(feature = "alloc")]
	fn copy_to_contiguous<'a>(mut self) -> impl Contiguous + 'a
	where
		Self: Sized,
	{
		let mut vec =
			alloc::vec::Vec::with_capacity(self.remaining());

		while self.has_remaining() {
			let chunk = self.chunk();
			vec.extend_from_slice(chunk);
			self.advance(chunk.len());
		}

		crate::Cursor::new(vec)
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

	/// Create a [`Reader`] adapter that implements [`std::io::Read`].
	#[cfg(feature = "std")]
	fn reader(self) -> Reader<Self>
	where
		Self: Sized,
	{
		Reader::new(self)
	}
}

macro_rules! forward_buf_impl {
	() => {
		fn chunk(&self) -> &[u8] {
			(**self).chunk()
		}

		fn remaining(&self) -> usize {
			(**self).remaining()
		}

		fn advance(&mut self, cnt: usize) {
			(**self).advance(cnt)
		}

		fn has_remaining(&self) -> bool {
			(**self).has_remaining()
		}

		fn copy_to_slice(&mut self, slice: &mut [u8]) -> usize {
			(**self).copy_to_slice(slice)
		}
	};
}

/// A non-contiguous buffer whose chunks can be iterated over.
pub trait Walkable: Buf {
	/// Chunk iterator type.
	type Iter<'a>: Iterator<Item = &'a [u8]>
	where
		Self: 'a;

	/// Walk each chunk of the buffer in order.
	///
	/// This function returns an [`Iterator`] that will iterate over all chunks
	/// of the buffer in order.
	fn walk_chunks(&self) -> Self::Iter<'_>;
}

#[rustfmt::skip]
macro_rules! forward_walkable_impl {
	($to:ty) => {
		type Iter<'a> = <$to>::Iter<'a>
		where
			Self: 'a;

		fn walk_chunks(&self) -> Self::Iter<'_> {
			(**self).walk_chunks()
		}
	};
}

/// A marker trait that describes the behavior of [`Buf::chunk()`].
///
/// # Safety
///
/// If this trait is implemented, then the slice returned by [`Buf::chunk()`] MUST
/// be of length [`Buf::remaining()`].
pub unsafe trait Contiguous: Buf + Walkable {}

// ========================================================

impl<B: Buf> Buf for &mut B {
	forward_buf_impl!();
}

impl<B: Walkable> Walkable for &mut B {
	forward_walkable_impl!(B);
}

unsafe impl<B: Contiguous> Contiguous for &mut B {}

impl Buf for &[u8] {
	fn chunk(&self) -> &[u8] {
		self
	}

	fn remaining(&self) -> usize {
		self.len()
	}

	fn advance(&mut self, cnt: usize) {
		assert!(
			cnt <= self.remaining(),
			"tried to advance past end of slice"
		);
		*self = &self[cnt..];
	}
}

impl Walkable for &[u8] {
	type Iter<'a> = Once<&'a [u8]>
	where
		Self: 'a;

	fn walk_chunks(&self) -> Self::Iter<'_> {
		once(*self)
	}
}

unsafe impl Contiguous for &[u8] {}

#[cfg(feature = "alloc")]
mod alloc_impls {
	use super::{Buf, Contiguous, Walkable};

	#[allow(unused_imports)]
	use alloc::boxed::Box;

	impl<B: Buf> Buf for Box<B> {
		forward_buf_impl!();
	}

	impl<B: Walkable> Walkable for Box<B> {
		forward_walkable_impl!(B);
	}

	unsafe impl<B: Contiguous> Contiguous for Box<B> {}
}

#[cfg(feature = "std")]
mod std_impls {
	use super::Buf;

	use std::io;

	/// An adapter for [`Buf`] that implements [`std::io::Read`].
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Reader<B>
	where
		B: Buf,
	{
		buf: B,
	}

	impl<B: Buf> Reader<B> {
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

		/// Destruct the [`Reader`] and get back the buffer.
		pub fn into_inner(self) -> B {
			self.buf
		}
	}

	impl<B: Buf> io::Read for Reader<B> {
		fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
			let n = self.buf.copy_to_slice(buf);
			Ok(n)
		}
	}
}

#[cfg(feature = "std")]
pub use self::std_impls::Reader;
