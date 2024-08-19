use core::fmt;

use crate::header::{Header, HeaderBytes};
use crate::payload::Payload;

use channels_io::buf::Buf;

/// An encoded [`Frame`].
///
/// This struct is a [`Buf`] that contains the encoded frame.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Encoded<T: AsRef<[u8]>> {
	header: HeaderBytes,
	payload: Payload<T>,
	pos: usize,
}

impl<T: AsRef<[u8]>> Encoded<T> {
	pub(super) fn new(header: Header, payload: Payload<T>) -> Self {
		Self { header: header.to_bytes(), payload, pos: 0 }
	}

	/// Get the length of the entire frame.
	#[inline]
	// `is_empty` doesn't really make sense within the context of `Encoded`. An
	// encoded frame is never 0 bytes in length.
	#[allow(clippy::len_without_is_empty)]
	pub fn len(&self) -> usize {
		self.header.len() + self.payload.as_slice().len()
	}
}

impl<T: AsRef<[u8]>> Buf for Encoded<T> {
	fn remaining(&self) -> usize {
		self.len() - self.pos
	}

	fn chunk(&self) -> &[u8] {
		let hdr = self.header.as_ref();
		let payload = self.payload.as_slice();

		if self.pos < hdr.len() {
			&hdr[self.pos..]
		} else {
			let pos = self.pos - hdr.len();
			&payload[pos..]
		}
	}

	fn advance(&mut self, n: usize) {
		assert!(n <= self.remaining(), "n must not be greater than the amount of remaining bytes");
		self.pos += n;
	}
}

impl<T: AsRef<[u8]>> fmt::Debug for Encoded<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Encoded")
			.field("header", &self.header)
			.field("payload", &self.payload)
			.field("pos", &self.pos)
			.finish()
	}
}
