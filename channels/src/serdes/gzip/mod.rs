//! A module containing the [`Gzip`] middleware.

use super::prelude::*;

use core::marker::PhantomData;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

// Reexport so users don't need to add flate2 to their dependencies
// just for this one type.
pub use flate2::Compression;

mod error;
pub use error::Error;

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
		Gzip { next, level: self.level, intermediate: Vec::new() }
	}
}

/// A middleware type which compresses/decompresses all data with gzip
/// using the [`mod@flate2`] crate.
#[derive(Debug, Clone)]
pub struct Gzip<U> {
	next: U,
	level: Compression,
	intermediate: Vec<u8>,
}

impl<U> Gzip<U> {
	/// Get a [`Builder`].
	pub fn builder() -> Builder<U> {
		Builder::default()
	}
}

impl<T, U> Serializer<T> for Gzip<U>
where
	U: Serializer<T>,
{
	type Error = Error<U::Error>;

	fn serialize<W: Write>(
		&mut self,
		mut buf: W,
		t: &T,
	) -> Result<(), Self::Error> {
		self.intermediate.clear();

		let compressed =
			GzEncoder::new(&mut self.intermediate, self.level);
		self.next.serialize(compressed, t).map_err(Error::Next)?;

		let length = self.intermediate.len() as u64;
		buf.write_all(&length.to_be_bytes()).map_err(Error::Io)?;
		buf.write_all(&self.intermediate).map_err(Error::Io)?;

		Ok(())
	}

	fn size_hint(&mut self, t: &T) -> Option<usize> {
		self.next.size_hint(t)
	}
}

impl<T, U> Deserializer<T> for Gzip<U>
where
	U: Deserializer<T>,
{
	type Error = Error<U::Error>;

	fn deserialize<R: Read>(
		&mut self,
		mut buf: R,
	) -> Result<T, Self::Error> {
		let length = {
			let mut length = [0u8; 8];
			buf.read_exact(&mut length).map_err(Error::Io)?;
			u64::from_be_bytes(length)
		};

		let buf = buf.take(length);

		let uncompressed = GzDecoder::new(buf);
		self.next.deserialize(uncompressed).map_err(Error::Next)
	}
}
