//! A module containing the [`Hmac`] middleware.

use super::prelude::*;

use core::marker::PhantomData;

use std::io;

use ::hmac::digest::OutputSizeUser;
use ::hmac::Mac;

mod error;
pub use error::Error;

/// [`Hmac`] builder.
#[derive(Debug, Clone)]
pub struct Builder<U, K>
where
	K: AsRef<[u8]>,
{
	_marker: PhantomData<Hmac<U, K>>,
	key: K,
}

impl<U, K> Builder<U, K>
where
	K: AsRef<[u8]>,
{
	/// Set the secret key used.
	pub fn key(mut self, key: K) -> Self {
		self.key = key;
		self
	}

	/// Build a [`Hmac`] structure.
	pub fn build(self, next: U) -> Hmac<U, K> {
		Hmac { next, key: self.key }
	}
}

/// A middleware type which cryptographically verifies all data
/// with [`HMAC-SHA3-512`](sha3::Sha3_512) using the [`mod@hmac`] crate.
#[derive(Debug, Clone)]
pub struct Hmac<U, K>
where
	K: AsRef<[u8]>,
{
	next: U,
	key: K,
}

impl<U, K> Hmac<U, K>
where
	K: AsRef<[u8]>,
{
	/// Get a [`Builder`].
	pub fn builder(key: K) -> Builder<U, K> {
		Builder { _marker: PhantomData, key }
	}

	pub(crate) fn create_hmac(&self) -> ::hmac::Hmac<sha3::Sha3_512> {
		::hmac::Hmac::<sha3::Sha3_512>::new_from_slice(
			self.key.as_ref(),
		)
		.unwrap()
	}
}

struct HmacRw<T, M>
where
	M: Mac,
{
	inner: T,
	mac: M,
}

impl<T, M> Write for HmacRw<T, M>
where
	T: Write,
	M: Mac,
{
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		let n = self.inner.write(buf)?;
		self.mac.update(&buf[..n]);
		Ok(n)
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}

impl<T, M> Read for HmacRw<T, M>
where
	T: Read,
	M: Mac,
{
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let n = self.inner.read(buf)?;
		self.mac.update(&buf[..n]);
		Ok(n)
	}
}

impl<T, U, K> Serializer<T> for Hmac<U, K>
where
	U: Serializer<T>,
	K: AsRef<[u8]>,
{
	type Error = Error<U::Error>;

	fn serialize<W: Write>(
		&mut self,
		mut buf: W,
		t: &T,
	) -> Result<(), Self::Error> {
		let mut writer =
			HmacRw { inner: &mut buf, mac: self.create_hmac() };

		self.next.serialize(&mut writer, t).map_err(Error::Next)?;

		let hmac_sign = writer.mac.finalize().into_bytes();
		buf.write_all(&hmac_sign).map_err(Error::Io)?;

		Ok(())
	}
}

impl<T, U, K> Deserializer<T> for Hmac<U, K>
where
	U: Deserializer<T>,
	K: AsRef<[u8]>,
{
	type Error = Error<U::Error>;

	fn deserialize<R: Read>(
		&mut self,
		mut buf: R,
	) -> Result<T, Self::Error> {
		let mut reader =
			HmacRw { inner: &mut buf, mac: self.create_hmac() };

		let res =
			self.next.deserialize(&mut reader).map_err(Error::Next);

		let sign_size = <::hmac::Hmac<sha3::Sha3_512> as OutputSizeUser>::output_size();

		let calculated = reader.mac.finalize().into_bytes();
		let mut unverified = vec![0u8; sign_size];
		buf.read_exact(&mut unverified).map_err(Error::Io)?;

		if unverified.as_slice() != calculated.as_slice() {
			return Err(Error::VerifyError);
		}

		res
	}
}
