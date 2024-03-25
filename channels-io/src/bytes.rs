/// A type that holds a contiguous slice of bytes.
pub trait AsBytes {
	/// Get the slice of bytes this type holds.
	fn as_bytes(&self) -> &[u8];
}

macro_rules! forward_as_bytes_impl {
	() => {
		fn as_bytes(&self) -> &[u8] {
			(**self).as_bytes()
		}
	};
}

/// A type that holds a contiguous slice of mutable bytes.
pub trait AsBytesMut: AsBytes {
	/// Get the mutable slice of bytes this type holds.
	fn as_bytes_mut(&mut self) -> &mut [u8];
}

macro_rules! forward_as_bytes_mut_impl {
	() => {
		fn as_bytes_mut(&mut self) -> &mut [u8] {
			(**self).as_bytes_mut()
		}
	};
}

// ========================================================

impl<B: AsBytes> AsBytes for &B {
	forward_as_bytes_impl!();
}

impl<B: AsBytes> AsBytes for &mut B {
	forward_as_bytes_impl!();
}

impl<B: AsBytesMut> AsBytesMut for &mut B {
	forward_as_bytes_mut_impl!();
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
	use super::{AsBytes, AsBytesMut};

	#[allow(unused_imports)]
	use alloc::{boxed::Box, vec::Vec};

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
