//! Generate IDs for use in headers.

use core::num::Wrapping;

/// An opaque type representing a packet ID.
///
/// This type is explicitly not [`Copy`] in order to avoid  accidental reuse of
/// the same ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id(u8);

impl Id {
	pub(crate) fn from_u8(value: u8) -> Self {
		Self(value)
	}

	pub(crate) fn as_u8(&self) -> u8 {
		self.0
	}
}

/// A never-ending sequence of packet IDs.
#[derive(Debug, Clone)]
pub struct IdSequence {
	next: Wrapping<u8>,
}

impl IdSequence {
	/// Create a new [`IdSequence`].
	#[must_use]
	pub fn new() -> Self {
		Self { next: Wrapping(0) }
	}

	/// Peek at the next [`Id`] in the sequence without advancing it.
	#[must_use]
	pub fn peek(&self) -> Id {
		Id::from_u8(self.next.0)
	}

	/// Get the next [`Id`] in the sequence and advance it.
	pub fn advance(&mut self) -> Id {
		let id = self.peek();
		self.next += 1;
		id
	}
}

impl Default for IdSequence {
	fn default() -> Self {
		Self::new()
	}
}
