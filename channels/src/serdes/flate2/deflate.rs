//! A module containing the [`Deflate`] middleware.

use super::prelude::*;

use core::marker::PhantomData;

use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;

// Reexport so users don't need to add flate2 to their dependencies
// just for this one type.
pub use flate2::Compression;

/// [`Deflate`] builder.
#[derive(Debug, Clone)]
pub struct Builder<U> {
	_marker: PhantomData<Deflate<U>>,
	level: Compression,
}

impl<U> Default for Builder<U> {
	fn default() -> Self {
		Self { _marker: PhantomData, level: Default::default() }
	}
}

impl<U> Builder<U> {
	/// Set the compression level.
	pub fn level(mut self, level: Compression) -> Self {
		self.level = level;
		self
	}

	/// Build a [`Deflate`] structure.
	pub fn build(self, next: U) -> Deflate<U> {
		Deflate { next, level: self.level }
	}
}

/// A middleware type which compresses/decompresses all data with deflate
/// using the [`mod@flate2`] crate.
#[derive(Debug, Clone)]
pub struct Deflate<U> {
	next: U,
	level: Compression,
}

impl<U> Deflate<U> {
	/// Get a [`Builder`].
	pub fn builder() -> Builder<U> {
		Builder::default()
	}
}

impl<T, U> Serializer<T> for Deflate<U>
where
	U: Serializer<T>,
{
	type Error = U::Error;

	fn serialize<W: Write>(
		&mut self,
		buf: W,
		t: &T,
	) -> Result<(), Self::Error> {
		let compressed = DeflateEncoder::new(buf, self.level);
		self.next.serialize(compressed, t)
	}

	fn size_hint(&mut self, t: &T) -> Option<usize> {
		self.next.size_hint(t)
	}
}

impl<T, U> Deserializer<T> for Deflate<U>
where
	U: Deserializer<T>,
{
	type Error = U::Error;

	fn deserialize<R: Read>(
		&mut self,
		buf: R,
	) -> Result<T, Self::Error> {
		let uncompressed = DeflateDecoder::new(buf);
		self.next.deserialize(uncompressed)
	}
}
