//! TODO: docs

use core::fmt;

/// TODO: docs
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

impl<T> fmt::Debug for PayloadError<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("PayloadError").finish()
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
	/// The maximum length, in bytes, of any payload.
	pub const MAX_LENGTH: usize = max_payload_length();

	/// TODO: docs
	///
	/// # Safety
	///
	/// The caller must ensure that the length of payload is not greater than
	/// [`Payload::MAX_LENGTH`].
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

	/// TODO: docs
	#[inline]
	pub fn as_ref(&self) -> Payload<&T> {
		Payload(&self.0)
	}

	/// TODO: docs
	#[inline]
	pub fn as_mut(&mut self) -> Payload<&mut T> {
		Payload(&mut self.0)
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
	#[allow(clippy::cast_possible_truncation)]
	pub fn length(&self) -> u32 {
		self.as_slice().len() as u32
	}
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for Payload<T> {
	fn as_ref(&self) -> &[u8] {
		self.as_slice()
	}
}

impl<T: AsRef<[u8]>> fmt::Debug for Payload<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_list().entries(self.as_slice()).finish()
	}
}

const fn max_payload_length() -> usize {
	let a = u32::MAX as u64;
	let b = usize::MAX as u64;

	#[allow(clippy::cast_possible_truncation)]
	if a <= b {
		a as usize
	} else {
		b as usize
	}
}
