//! TODO: docs

use core::fmt;

use crate::num::u48;

/// TODO: docs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PayloadError<T> {
	payload: T,
}

impl<T> PayloadError<T> {
	/// TODO: docs
	#[inline]
	pub fn into_payload(self) -> T {
		self.payload
	}
}

impl<T> fmt::Display for PayloadError<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("payload too large")
	}
}

#[cfg(feature = "std")]
impl<T> std::error::Error for PayloadError<T> {}

/// TODO: docs
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Payload<T>(T);

impl<T> Payload<T> {
	const MAX_LENGTH: usize = max_payload_length();

	/// TODO: docs
	#[inline]
	pub const unsafe fn new_unchecked(payload: T) -> Self {
		Self(payload)
	}

	/// TODO: docs
	#[inline]
	pub fn get(&self) -> &T {
		&self.0
	}

	/// TODO: docs
	#[inline]
	pub fn get_mut(&mut self) -> &mut T {
		&mut self.0
	}

	/// TODO: docs
	#[inline]
	pub fn into_inner(self) -> T {
		self.0
	}
}

impl<T: AsRef<[u8]>> Payload<T> {
	/// TODO: docs
	#[inline]
	pub fn new(payload: T) -> Result<Self, PayloadError<T>> {
		if payload.as_ref().len() > Self::MAX_LENGTH {
			Err(PayloadError { payload })
		} else {
			Ok(Self(payload))
		}
	}

	/// TODO: docs
	#[inline]
	pub fn as_slice(&self) -> &[u8] {
		self.0.as_ref()
	}

	/// TODO: docs
	#[inline]
	pub fn length(&self) -> u48 {
		u48::new_truncate(self.as_slice().len() as u64)
	}
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for Payload<T> {
	fn as_ref(&self) -> &[u8] {
		self.as_slice()
	}
}

impl<T: AsRef<[u8]>> fmt::Debug for Payload<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		struct DebugHex<'a>(&'a [u8]);

		impl fmt::Debug for DebugHex<'_> {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				write!(f, "{:02x?}", self.0)
			}
		}

		f.debug_tuple("Payload")
			.field(&DebugHex(self.as_slice()))
			.finish()
	}
}

const fn max_payload_length() -> usize {
	let a = u48::MAX.get();
	let b = usize::MAX as u64;

	#[allow(clippy::cast_possible_truncation)]
	if a <= b {
		a as usize
	} else {
		b as usize
	}
}
