//! Middleware that verifies data with a CRC checksum.

use core::fmt;

extern crate alloc;

use alloc::boxed::Box;

use channels_io::{Buf, Contiguous, Cursor, Walkable};

use crate::{Deserializer, Serializer};

/// Middleware that verifies data with a CRC checksum.
///
/// When working as a [`Serializer`], it simply computes an 8 byte CRC checksum
/// of the data it was given and returns the original data with the checksum
/// appended to the end in big-endian format. When working as a [`Deserializer`],
/// it reads the checksum of the data (the last 8 bytes), computes the checksum
/// again from the read data and then compares the 2 checksums. If don't match,
/// the [`Deserializer::deserialize()`] returns with [`Err(DeserializeError::InvalidChecksum)`].
/// If the 2 checksums match, the data is then given to the next deserialize in
/// the chain. Any errors from the next deserializer are returned via [`Err(DeserializeError::Next)`].
///
/// [`Err(DeserializeError::InvalidChecksum)`]: DeserializeError::InvalidChecksum
/// [`Err(DeserializeError::Next)`]: DeserializeError::Next
pub struct Crc<U> {
	next: U,
	crc: Box<crc::Crc<u64>>,
}

impl<U> fmt::Debug for Crc<U>
where
	U: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Crc")
			.field("next", &self.next)
			.finish_non_exhaustive()
	}
}

impl<U> Clone for Crc<U>
where
	U: Clone,
{
	fn clone(&self) -> Self {
		Self::new(self.next.clone(), self.crc.algorithm)
	}
}

impl<U> Default for Crc<U>
where
	U: Default,
{
	fn default() -> Self {
		Self::new(Default::default(), Self::DEFAULT_ALGORITHM)
	}
}

impl<U> Crc<U> {
	const DEFAULT_ALGORITHM: &'static crc::Algorithm<u64> =
		&crc::CRC_64_XZ;

	/// Create a new [`Crc`] middleware.
	pub fn new(
		next: U,
		algorithm: &'static crc::Algorithm<u64>,
	) -> Self {
		Self { next, crc: Box::new(crc::Crc::<u64>::new(algorithm)) }
	}

	/// Get a reference to the next serializer in the chain.
	pub fn next_ref(&self) -> &U {
		&self.next
	}

	/// Get a reference to the next serializer in the chain.
	pub fn next_mut(&mut self) -> &mut U {
		&mut self.next
	}

	/// Consume `self` and return the next serializer in the chain.
	pub fn into_next(self) -> U {
		self.next
	}
}

impl<T, U> Serializer<T> for Crc<U>
where
	U: Serializer<T>,
{
	type Error = U::Error;

	fn serialize(
		&mut self,
		t: &T,
	) -> Result<impl Walkable, Self::Error> {
		let data = self.next.serialize(t)?;

		let mut digest = self.crc.digest();
		data.walk_chunks().for_each(|chunk| digest.update(chunk));
		let checksum = digest.finalize();

		let checksum = Cursor::new(checksum.to_be_bytes());
		let output = data.chain(checksum);
		Ok(output)
	}
}

/// Possible errors that might occur during deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeserializeError<T> {
	/// The data could not be verified because the checksum is not correct.
	InvalidChecksum,
	/// No checksum exists in the data.
	NoChecksum,
	/// An error from the next deserializer in the chain.
	Next(T),
}

impl<T, U> Deserializer<T> for Crc<U>
where
	U: Deserializer<T>,
{
	type Error = DeserializeError<U::Error>;

	fn deserialize(
		&mut self,
		buf: impl Contiguous,
	) -> Result<T, Self::Error> {
		let inner_len = buf
			.remaining()
			.checked_sub(8)
			.ok_or(DeserializeError::NoChecksum)?;

		let (inner, checksum) = buf.chunk().split_at(inner_len);

		let unverified = u64::from_be_bytes(checksum.try_into().expect(
			"remaining part of payload should have been at least 8 bytes",
		));

		let mut digest = self.crc.digest();
		digest.update(inner);
		let calculated = digest.finalize();

		if unverified != calculated {
			return Err(DeserializeError::InvalidChecksum);
		}

		let t = self
			.next
			.deserialize(inner)
			.map_err(DeserializeError::Next)?;

		Ok(t)
	}
}
