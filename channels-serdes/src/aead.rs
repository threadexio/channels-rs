//! Middleware that encrypts data for transport.

use core::fmt;

use channels_io::{Buf, Contiguous, Cursor, Walkable};

use ring::aead::{self, Aad};

use crate::{Deserializer, Serializer};

/// Algorithms usable with this middleware.
///
/// This module reexports the algorithms from [`ring::aead`].
pub mod algorithm {
	pub use super::aead::{
		AES_128_GCM, AES_256_GCM, CHACHA20_POLY1305,
	};
}

pub use self::aead::{
	BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey,
	UnboundKey,
};

/// Middleware that encrypts data for transport.
///
/// This is the serializer.
#[derive(Debug)]
pub struct Encrypt<U, N>
where
	N: NonceSequence,
{
	next: U,
	key: SealingKey<N>,
}

impl<U, N> Encrypt<U, N>
where
	N: NonceSequence,
{
	/// Create a new [`Encrypt`] serializer that uses `key`.
	pub fn new(next: U, key: SealingKey<N>) -> Self {
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

/// Possible errors that might occur during serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializeError<T> {
	/// An error from the next serializer in the chain.
	Next(T),
	/// The data could not be encrypted.
	EncryptError,
}

impl<T> fmt::Display for SerializeError<T>
where
	T: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Next(e) => e.fmt(f),
			Self::EncryptError => f.write_str("encryption failure"),
		}
	}
}

#[cfg(feature = "std")]
impl<T: std::error::Error> std::error::Error for SerializeError<T> {}

impl<T, U, N> Serializer<T> for Encrypt<U, N>
where
	N: NonceSequence,
	U: Serializer<T>,
{
	type Error = SerializeError<U::Error>;

	fn serialize(
		&mut self,
		t: &T,
	) -> Result<impl Walkable, Self::Error> {
		let data =
			self.next.serialize(t).map_err(SerializeError::Next)?;

		let mut out = copy_buf_to_vec(data);

		let tag = self
			.key
			.seal_in_place_separate_tag(Aad::empty(), &mut out)
			.map_err(|_| SerializeError::EncryptError)?;

		let out = Cursor::new(out);
		let tag = Cursor::new(tag.as_ref().to_vec());

		let out = out.chain(tag);
		Ok(out)
	}
}

/// Middleware that encrypts data for transport.
///
/// This is the deserializer.
#[derive(Debug)]
pub struct Decrypt<U, N>
where
	N: NonceSequence,
{
	next: U,
	key: OpeningKey<N>,
}

impl<U, N> Decrypt<U, N>
where
	N: NonceSequence,
{
	/// Create a new [`Decrypt`] serializer that uses `key`.
	pub fn new(next: U, key: OpeningKey<N>) -> Self {
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

/// Possible errors that might occur during deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeserializeError<T> {
	/// An error from the next deserializer in the chain.
	Next(T),
	/// The data does not have a tag.
	NoTag,
	/// The data could not be decrypted.
	DecryptError,
}

impl<T> fmt::Display for DeserializeError<T>
where
	T: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Next(e) => e.fmt(f),
			Self::NoTag => f.write_str("no tag"),
			Self::DecryptError => f.write_str("decryption failure"),
		}
	}
}

#[cfg(feature = "std")]
impl<T: std::error::Error> std::error::Error for DeserializeError<T> {}

impl<T, U, N> Deserializer<T> for Decrypt<U, N>
where
	N: NonceSequence,
	U: Deserializer<T>,
{
	type Error = DeserializeError<U::Error>;

	fn deserialize(
		&mut self,
		buf: impl Contiguous,
	) -> Result<T, Self::Error> {
		let mut input = copy_buf_to_vec(buf);

		let plaintext = self
			.key
			.open_in_place(Aad::empty(), &mut input)
			.map_err(|_| DeserializeError::DecryptError)?;

		self.next
			.deserialize(plaintext.as_ref())
			.map_err(DeserializeError::Next)
	}
}

fn copy_buf_to_vec<B: Buf>(mut buf: B) -> alloc::vec::Vec<u8> {
	let mut out = Vec::with_capacity(buf.remaining());

	while buf.has_remaining() {
		let chunk = buf.chunk();
		out.extend_from_slice(chunk);
		buf.advance(chunk.len());
	}

	out
}
