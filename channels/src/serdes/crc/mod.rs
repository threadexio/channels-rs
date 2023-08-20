//! A module containing the [`Crc`] middleware.

use super::prelude::*;

use core::marker::PhantomData;
use core::mem::size_of;

use std::io;

mod error;
pub use error::Error;

pub mod algorithm;
use algorithm::Width;

/// [`Crc`] builder.
#[derive(Clone)]
pub struct Builder<U> {
	_marker: PhantomData<U>,
	algorithm: &'static crc::Algorithm<Width>,
}

impl<U> Default for Builder<U> {
	fn default() -> Self {
		Self {
			_marker: Default::default(),
			algorithm: &crc::CRC_32_BZIP2,
		}
	}
}

impl<U> Builder<U> {
	/// Set the algorithm used.
	pub fn algorithm(
		mut self,
		algorithm: &'static crc::Algorithm<Width>,
	) -> Self {
		self.algorithm = algorithm;
		self
	}

	/// Build a [`Crc`] structure.
	pub fn build(self, next: U) -> Crc<U> {
		Crc { next, algorithm: self.algorithm }
	}
}

/// A middleware type which stores a `crc` field at the end of the
/// data to validate that it was transported correctly.
pub struct Crc<U> {
	next: U,
	algorithm: &'static crc::Algorithm<Width>,
}

impl<U> Crc<U> {
	/// Get a [`Builder`].
	pub fn builder() -> Builder<U> {
		Builder::default()
	}
}

impl<U> Clone for Crc<U>
where
	U: Clone,
{
	fn clone(&self) -> Self {
		Self {
			next: self.next.clone(),
			algorithm: self.algorithm.clone(),
		}
	}
}

impl<T, U> Serializer<T> for Crc<U>
where
	U: Serializer<T>,
{
	type Error = Error<U::Error>;

	fn serialize<W: Write>(
		&mut self,
		mut buf: W,
		t: &T,
	) -> Result<(), Self::Error> {
		let c: crc::Crc<u32> =
			crc::Crc::<u32>::new(&crc::CRC_32_BZIP2);
		let mut writer =
			CrcRw { inner: &mut buf, digest: c.digest() };

		self.next.serialize(&mut writer, t).map_err(Error::Next)?;

		let checksum = writer.digest.finalize();
		buf.write_all(&checksum.to_be_bytes()).map_err(Error::Io)?;

		Ok(())
	}

	fn size_hint(&mut self, t: &T) -> Option<usize> {
		let s = self.next.size_hint(t)? + size_of::<Width>();
		Some(s)
	}
}

impl<T, U> Deserializer<T> for Crc<U>
where
	U: Deserializer<T>,
{
	type Error = Error<U::Error>;

	fn deserialize<R: Read>(
		&mut self,
		mut buf: R,
	) -> Result<T, Self::Error> {
		let c = crc::Crc::<u32>::new(&crc::CRC_32_BZIP2);
		let mut reader =
			CrcRw { inner: &mut buf, digest: c.digest() };

		let res =
			self.next.deserialize(&mut reader).map_err(Error::Next);

		let mut unverified = [0u8; 4];
		reader.read_exact(&mut unverified).map_err(Error::Io)?;
		let unverified = u32::from_be_bytes(unverified);

		let computed = reader.digest.finalize();

		if unverified != computed {
			return Err(Error::ChecksumError);
		}

		res
	}
}

struct CrcRw<'a, T> {
	inner: T,
	digest: crc::Digest<'a, Width>,
}

impl<T> Write for CrcRw<'_, T>
where
	T: Write,
{
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		let n = self.inner.write(buf)?;
		self.digest.update(&buf[..n]);
		Ok(n)
	}

	fn flush(&mut self) -> io::Result<()> {
		self.inner.flush()
	}
}

impl<T> Read for CrcRw<'_, T>
where
	T: Read,
{
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let n = self.inner.read(buf)?;
		self.digest.update(&buf[..n]);
		Ok(n)
	}
}
