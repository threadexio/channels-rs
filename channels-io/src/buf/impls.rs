use core::iter::{once, Once};

use super::{
	AsBytes, AsBytesMut, Buf, BufMut, Contiguous, ContiguousMut,
	Walkable, WalkableMut,
};

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

macro_rules! forward_walkable_impl {
    ($to:ty, $($lifetime:tt)+) => {
        type Iter = <$to>::Iter;

        fn walk_chunks(&$($lifetime)+ self) -> Self::Iter {
            (**self).walk_chunks()
        }
    };
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

macro_rules! forward_walkable_mut_impl {
    ($to:ty, $($lifetime:tt)+) => {
        type Iter = <$to>::Iter;

        fn walk_chunks_mut(&$($lifetime)+ mut self) -> Self::Iter {
            (**self).walk_chunks_mut()
        }
    };
}

macro_rules! forward_as_bytes_impl {
	() => {
		fn as_bytes(&self) -> &[u8] {
			(**self).as_bytes()
		}
	};
}

macro_rules! forward_as_bytes_mut_impl {
	() => {
		fn as_bytes_mut(&mut self) -> &mut [u8] {
			(**self).as_bytes_mut()
		}
	};
}

use forward_buf_impl;
use forward_walkable_impl;

use forward_bufmut_impl;
use forward_walkable_mut_impl;

use forward_as_bytes_impl;
use forward_as_bytes_mut_impl;

impl<B: Buf> Buf for &mut B {
	forward_buf_impl!();
}
unsafe impl<B: Contiguous> Contiguous for &mut B {}
impl<'a, B: Walkable<'a>> Walkable<'a> for &mut B {
	forward_walkable_impl!(B, 'a);
}

impl<B: BufMut> BufMut for &mut B {
	forward_bufmut_impl!();
}
unsafe impl<B: ContiguousMut> ContiguousMut for &mut B {}
impl<'a, B: WalkableMut<'a>> WalkableMut<'a> for &mut B {
	forward_walkable_mut_impl!(B, 'a);
}

impl<B: AsBytes> AsBytes for &B {
	forward_as_bytes_impl!();
}

impl<B: AsBytes> AsBytes for &mut B {
	forward_as_bytes_impl!();
}
impl<B: AsBytesMut> AsBytesMut for &mut B {
	forward_as_bytes_mut_impl!();
}

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

unsafe impl Contiguous for &[u8] {}

impl<'a> Walkable<'a> for &[u8] {
	type Iter = Once<&'a [u8]>;

	fn walk_chunks(&'a self) -> Self::Iter {
		once(*self)
	}
}

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

unsafe impl ContiguousMut for &mut [u8] {}

impl<'a> WalkableMut<'a> for &mut [u8] {
	type Iter = Once<&'a mut [u8]>;

	fn walk_chunks_mut(&'a mut self) -> Self::Iter {
		once(*self)
	}
}

impl<const N: usize> AsBytes for [u8; N] {
	fn as_bytes(&self) -> &[u8] {
		self.as_slice()
	}
}

impl<const N: usize> AsBytesMut for [u8; N] {
	fn as_bytes_mut(&mut self) -> &mut [u8] {
		self.as_mut_slice()
	}
}

#[cfg(feature = "alloc")]
mod alloc_impls {
	use super::*;

	#[allow(unused_imports)]
	use alloc::{boxed::Box, vec::Vec};

	impl<B: Buf> Buf for Box<B> {
		forward_buf_impl!();
	}
	unsafe impl<B: Contiguous> Contiguous for Box<B> {}
	impl<'a, B: Walkable<'a>> Walkable<'a> for Box<B> {
		forward_walkable_impl!(B, 'a);
	}

	impl<B: BufMut> BufMut for Box<B> {
		forward_bufmut_impl!();
	}
	unsafe impl<B: ContiguousMut> ContiguousMut for Box<B> {}
	impl<'a, B: WalkableMut<'a>> WalkableMut<'a> for Box<B> {
		forward_walkable_mut_impl!(B, 'a);
	}

	impl<B: AsBytes> AsBytes for Box<B> {
		forward_as_bytes_impl!();
	}

	impl<B: AsBytesMut> AsBytesMut for Box<B> {
		forward_as_bytes_mut_impl!();
	}

	impl AsBytes for Vec<u8> {
		fn as_bytes(&self) -> &[u8] {
			self.as_slice()
		}
	}

	impl AsBytesMut for Vec<u8> {
		fn as_bytes_mut(&mut self) -> &mut [u8] {
			self.as_mut_slice()
		}
	}
}
