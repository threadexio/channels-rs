//! Middleware that compresses data with DEFLATE.

use core::fmt;

use std::io::{Read, Write};

use channels_io::{Contiguous, Cursor, Walkable};
use flate2::{read::DeflateDecoder, write::DeflateEncoder};

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

#[cfg(feature = "std")]
impl<T: std::error::Error> std::error::Error for SerializeError<T> {}

impl<T, U> Serializer<T> for Deflate<U>
where
	U: Serializer<T>,
{
	type Error = SerializeError<U::Error>;

	fn serialize(
		&mut self,
		t: &T,
	) -> Result<impl Walkable, Self::Error> {
		let data =
			self.next.serialize(t).map_err(SerializeError::Next)?;

		let mut encoder = DeflateEncoder::new(Vec::new(), self.level);

		data.walk_chunks()
			.try_for_each(|chunk| encoder.write_all(chunk))
			.map_err(SerializeError::EncodeError)?;

		let output =
			encoder.finish().map_err(SerializeError::EncodeError)?;

		let output = Cursor::new(output);
		Ok(output)
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

#[cfg(feature = "std")]
impl<T: std::error::Error> std::error::Error for DeserializeError<T> {}

impl<T, U> Deserializer<T> for Deflate<U>
where
	U: Deserializer<T>,
{
	type Error = DeserializeError<U::Error>;

	fn deserialize(
		&mut self,
		buf: impl Contiguous,
	) -> Result<T, Self::Error> {
		let mut decoder = DeflateDecoder::new(buf.reader());

		let mut output = Vec::new();
		decoder
			.read_to_end(&mut output)
			.map_err(DeserializeError::DecodeError)?;

		let output = Cursor::new(output);
		self.next
			.deserialize(output)
			.map_err(DeserializeError::Next)
	}
}
