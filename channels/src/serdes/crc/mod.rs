//! A module containing the [`Crc`] middleware.

use super::prelude::*;

use core::fmt;
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
#[derive(Clone)]
pub struct Crc<U> {
	next: U,
	algorithm: &'static crc::Algorithm<Width>,
}

impl<U> Crc<U> {
	/// Get a [`Builder`].
	pub fn builder() -> Builder<U> {
		Builder::default()
	}

	pub(crate) fn create_crc(&self) -> crc::Crc<Width> {
		crc::Crc::<Width>::new(self.algorithm)
	}
}

impl<U> fmt::Debug for Crc<U>
where
	U: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Crc")
			.field("next", &self.next)
			.field("algorithm", &"...")
			.finish()
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
		let c = self.create_crc();

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
		let c = self.create_crc();

		let mut reader =
			CrcRw { inner: &mut buf, digest: c.digest() };

		let res =
			self.next.deserialize(&mut reader).map_err(Error::Next);

		let calculated = reader.digest.finalize();

		let unverified = {
			let mut unverified = [0u8; 4];
			buf.read_exact(&mut unverified).map_err(Error::Io)?;
			u32::from_be_bytes(unverified)
		};

		if unverified != calculated {
			return Err(Error::ChecksumError);
		}

		res
	}
}
