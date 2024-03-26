//! Middleware that verifies data with HMAC.

use core::fmt;

use channels_io::{Buf, Contiguous, Cursor, Walkable};

use ring::hmac;

use crate::{Deserializer, Serializer};

/// Algorithms usable with [`Key`].
///
/// This module reexports the algorithms from [`ring::hmac`].
pub mod algorithms {
	pub use super::hmac::{
		HMAC_SHA1_FOR_LEGACY_USE_ONLY, HMAC_SHA256, HMAC_SHA384,
		HMAC_SHA512,
	};
}

pub use self::hmac::Key;

/// Middleware that verifies data with HMAC.
#[derive(Debug, Clone)]
pub struct Hmac<U> {
	next: U,
	key: Key,
}

impl<U> Hmac<U> {
	/// Create a new [`Hmac`] middleware that uses `key`.
	pub fn new(next: U, key: Key) -> Self {
		Self { next, key }
	}

	/// Get a reference to the next serializer/deserializer in the chain.
	pub fn next_ref(&self) -> &U {
		&self.next
	}

	/// Get a reference to the next serializer/deserializer in the chain.
	pub fn next_mut(&mut self) -> &mut U {
		&mut self.next
	}

	/// Consume `self` and return the next serializer/deserializer in the chain.
	pub fn into_next(self) -> U {
		self.next
	}
}

impl<T, U> Serializer<T> for Hmac<U>
where
	U: Serializer<T>,
{
	type Error = U::Error;

	fn serialize(
		&mut self,
		t: &T,
	) -> Result<impl Walkable, Self::Error> {
		let data = self.next.serialize(t)?;

		let mut ctx = hmac::Context::with_key(&self.key);
		data.walk_chunks().for_each(|chunk| ctx.update(chunk));

		let tag = Cursor::new(ctx.sign().as_ref().to_vec());
		let output = data.chain(tag);

		Ok(output)
	}
}

/// Possible errors that might occur during deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeserializeError<T> {
	/// The data could not be verified because the HMAC does not match.
	VerifyFail,
	/// The data does not have a tag.
	NoTag,
	/// An error from the next deserializer in the chain.
	Next(T),
}

impl<T> fmt::Display for DeserializeError<T>
where
	T: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Next(e) => e.fmt(f),
			Self::NoTag => f.write_str("no tag"),
			Self::VerifyFail => f.write_str("verification failure"),
		}
	}
}

#[cfg(feature = "std")]
impl<T: std::error::Error> std::error::Error for DeserializeError<T> {}

impl<T, U> Deserializer<T> for Hmac<U>
where
	U: Deserializer<T>,
{
	type Error = DeserializeError<U::Error>;

	fn deserialize(
		&mut self,
		buf: impl Contiguous,
	) -> Result<T, Self::Error> {
		let tag_len =
			self.key.algorithm().digest_algorithm().output_len();
		let buf = buf.chunk();

		let tag_start = buf
			.len()
			.checked_sub(tag_len)
			.ok_or(DeserializeError::NoTag)?;
		let (data, tag) = buf.split_at(tag_start);

		hmac::verify(&self.key, data, tag)
			.map_err(|_| DeserializeError::VerifyFail)?;

		self.next.deserialize(data).map_err(DeserializeError::Next)
	}
}
