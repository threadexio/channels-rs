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
pub struct Builder<U> {
	_marker: PhantomData<Gzip<U>>,
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

	/// Build a [`Gzip`] structure.
	pub fn build(self, next: U) -> Gzip<U> {
		Gzip { next, level: self.level }
	}
}

/// A middleware type which compresses/decompresses all data with gzip
/// using the [`mod@flate2`] crate.
#[derive(Debug)]
pub struct Gzip<U> {
	next: U,
	level: Compression,
}

impl<U> Gzip<U> {
	/// Get a [`Builder`].
	pub fn builder() -> Builder<U> {
		Builder::default()
	}
}

impl<U> Clone for Gzip<U>
where
	U: Clone,
{
	fn clone(&self) -> Self {
		Self { next: self.next.clone(), level: self.level }
	}
}

impl<T, U> Serializer<T> for Gzip<U>
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

impl<T, U> Deserializer<T> for Gzip<U>
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
