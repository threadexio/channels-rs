use core::ops;

use super::Cursor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedBuf {
	inner: Cursor<Vec<u8>>,
}

impl OwnedBuf {
	pub fn new(capacity: usize) -> Self {
		Self { inner: Cursor::new(vec![0u8; capacity]) }
	}
}

impl ops::Deref for OwnedBuf {
	type Target = Cursor<Vec<u8>>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl ops::DerefMut for OwnedBuf {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

#[derive(Debug, PartialEq, Eq)]
pub struct BorrowedBuf<'a> {
	inner: Cursor<&'a mut [u8]>,
}

impl<'a> BorrowedBuf<'a> {
	pub fn new(slice: &'a mut [u8]) -> Self {
		Self { inner: Cursor::new(slice) }
	}
}

impl<'a> ops::Deref for BorrowedBuf<'a> {
	type Target = Cursor<&'a mut [u8]>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<'a> ops::DerefMut for BorrowedBuf<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}
