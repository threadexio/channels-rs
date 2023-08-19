//! A module containing the [`Gzip`] middleware.

use super::prelude::*;

use core::marker::PhantomData;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

// Reexport so users don't need to add flate2 to their dependencies
// just for this one type.
pub use flate2::Compression;

/// [`Gzip`] builder.
#[derive(Debug, Clone)]
pub struct Builder<T, U> {
	_marker: PhantomData<Gzip<T, U>>,
	level: Compression,
}

impl<T, U> Default for Builder<T, U> {
	fn default() -> Self {
		Self { _marker: PhantomData, level: Default::default() }
	}
}

impl<T, U> Builder<T, U> {
	/// Set the compression level.
	pub fn level(mut self, level: Compression) -> Self {
		self.level = level;
		self
	}

	/// Build a [`Gzip`] structure.
	pub fn build(self, next: U) -> Gzip<T, U> {
		Gzip { _marker: PhantomData, next, level: self.level }
	}
}

/// A middleware type which compresses/decompresses all data with gzip
/// using the [`mod@flate2`] crate.
#[derive(Debug)]
pub struct Gzip<T, U> {
	_marker: PhantomData<T>,
	next: U,
	level: Compression,
}

impl<T, U> Gzip<T, U> {
	/// Get a [`Builder`].
	pub fn builder() -> Builder<T, U> {
		Builder::default()
	}
}

impl<T, U> Clone for Gzip<T, U>
where
	U: Clone,
{
	fn clone(&self) -> Self {
		Self {
			_marker: self._marker,
			next: self.next.clone(),
			level: self.level,
		}
	}
}

impl<T, U> Serializer<T> for Gzip<T, U>
where
	U: Serializer<T>,
{
	type Error = U::Error;

	fn serialize<W: Write>(
		&mut self,
		buf: W,
		t: &T,
	) -> Result<(), Self::Error> {
		let compressed = GzEncoder::new(buf, self.level);
		self.next.serialize(compressed, t)
	}

	fn size_hint(&mut self, t: &T) -> Option<usize> {
		self.next.size_hint(t)
	}
}

impl<T, U> Deserializer<T> for Gzip<T, U>
where
	U: Deserializer<T>,
{
	type Error = U::Error;

	fn deserialize<R: Read>(
		&mut self,
		buf: R,
	) -> Result<T, Self::Error> {
		let uncompressed = GzDecoder::new(buf);
		self.next.deserialize(uncompressed)
	}
}
