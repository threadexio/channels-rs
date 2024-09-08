//! Middleware that compresses data with DEFLATE.

use core::fmt;

use std::io::{Read, Write};

use flate2::{read::DeflateDecoder, write::DeflateEncoder};

use crate::util::Error;
use crate::{Deserializer, Serializer};

pub use flate2::Compression;

/// Middleware that compresses data with DEFLATE.
#[derive(Debug, Clone)]
pub struct Deflate<U> {
	next: U,
	level: Compression,
}

impl<U> Default for Deflate<U>
where
	U: Default,
{
	fn default() -> Self {
		Self::new(Default::default(), Compression::fast())
	}
}

impl<U> Deflate<U> {
	/// Create a new [`Deflate`] middleware.
	pub fn new(next: U, level: Compression) -> Self {
		Self { next, level }
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
#[derive(Debug)]
pub enum SerializeError<T> {
	/// An error from the next serializer in the chain.
	Next(T),
	/// The encoder failed to encode a part of the payload.
	EncodeError(std::io::Error),
}

impl<T> fmt::Display for SerializeError<T>
where
	T: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Next(e) => e.fmt(f),
			Self::EncodeError(e) => {
				f.write_str("encode error: ")?;
				e.fmt(f)
			},
		}
	}
}

impl<T: Error> Error for SerializeError<T> {}

impl<T, U> Serializer<T> for Deflate<U>
where
	U: Serializer<T>,
{
	type Error = SerializeError<U::Error>;

	fn serialize(&mut self, t: &T) -> Result<Vec<u8>, Self::Error> {
		let data =
			self.next.serialize(t).map_err(SerializeError::Next)?;

		let mut encoder = DeflateEncoder::new(Vec::new(), self.level);

		encoder
			.write_all(&data)
			.map_err(SerializeError::EncodeError)?;

		let out =
			encoder.finish().map_err(SerializeError::EncodeError)?;

		Ok(out)
	}
}

/// Possible errors that might occur during deserialization.
#[derive(Debug)]
pub enum DeserializeError<T> {
	/// An error from the next deserializer in the chain.
	Next(T),
	/// The decoder failed to decode a part of the payload.
	DecodeError(std::io::Error),
}

impl<T> fmt::Display for DeserializeError<T>
where
	T: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Next(e) => e.fmt(f),
			Self::DecodeError(e) => {
				f.write_str("decode error: ")?;
				e.fmt(f)
			},
		}
	}
}

impl<T: Error> Error for DeserializeError<T> {}

impl<T, U> Deserializer<T> for Deflate<U>
where
	U: Deserializer<T>,
{
	type Error = DeserializeError<U::Error>;

	fn deserialize(
		&mut self,
		buf: &mut [u8],
	) -> Result<T, Self::Error> {
		let mut decoder = DeflateDecoder::new(buf as &[u8]);

		let mut out = Vec::new();
		decoder
			.read_to_end(&mut out)
			.map_err(DeserializeError::DecodeError)?;

		self.next
			.deserialize(&mut out)
			.map_err(DeserializeError::Next)
	}
}
