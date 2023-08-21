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
///
/// # Security
///
/// This middleware is **NOT** a silver bullet. It does protect against
/// modification of the data but **not** against [replay attacks](https://csrc.nist.gov/glossary/term/replay_attack).
///
/// To understand why this is, one must now think what HMAC is. It is a way
/// for 2 parties to verify that some piece of data is authenticated and
/// not tampered with in any way.
///
/// ## A quick explanation
///
/// Think of HMAC as a hand-written signature; it does verify the document
/// but it does not prevent anyone from photocopying the whole document
/// with the valid signature on it. This is rather problematic in certain
/// cases where we also need to guarantee the uniqueness of that document.
/// This is tackled with bundling in a [nonce value](https://csrc.nist.gov/glossary/term/nonce)
/// which the other party must be able to verify on their end. This way even
/// if an attacker were to copy the data byte-for-byte, that data would then
/// get rejected because its nonce value has already been used previously.
/// And they could not just change the nonce value because then the signature
/// would not match and they would not have the secret key to create a new
/// signature.
///
/// ## Why does this crate not support this?
///
/// Nonce values are supposed to be unique. In other protocols, an initial state
/// is shared by the 2 parties in the initial handshake process after they
/// establish a secure channel. That initial state can then be used to seed a
/// [CSRNG](https://en.wikipedia.org/wiki/Cryptographically_secure_pseudorandom_number_generator)
/// which can then be used to generate nonce values.
///
/// The protocol this crate uses does not support any handshaking process on its
/// own, but rather uses the fact that software using this crate is usually
/// compiled together or have access to the same secret key at compile-time.
/// This means that a handshake is not required to exchange keys. It also means
/// that if we shared that initial state, every time the software ran it would
/// use the exact same nonce values in the same order, thus rendering the entire
/// system useless. An attacker could simply capture the traffic from one execution
/// and replay it later when the software restarts.
///
/// ## Mitigation
///
/// If you find yourself needing data to be signed and/or encrypted, you might find
/// a use for a crate like [`rustls`](https://github.com/rustls/rustls) and use the
/// [`io::Read`] and [`io::Write`] types it provides without this middleware.
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
