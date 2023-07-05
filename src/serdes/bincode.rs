use super::impl_prelude::*;

use bincode::{ErrorKind, Options};
macro_rules! bincode {
	() => {
		::bincode::options()
			.reject_trailing_bytes()
			.with_big_endian()
			.with_fixint_encoding()
			.with_no_limit()
	};
}

/// The [`bincode`] serializer which automatically works with all
/// types that implement `serde`'s `Serialize` and `Deserialize`.
pub struct Bincode;

impl<T> Serializer<T> for Bincode
where
	T: serde::Serialize,
{
	fn serialize(&mut self, t: &T) -> Result<Vec<u8>, Error> {
		match bincode!().serialize(t) {
			Ok(v) => Ok(v),
			Err(e) => Err(Error::Other(e.to_string())),
		}
	}
}

impl<T> Deserializer<T> for Bincode
where
	for<'de> T: serde::Deserialize<'de>,
{
	fn deserialize(&mut self, buf: &[u8]) -> Result<T, Error> {
		match bincode!().deserialize(buf) {
			Ok(v) => Ok(v),
			Err(e) => match *e {
				ErrorKind::SizeLimit => Err(Error::NotEnough),
				_ => Err(Error::Other(e.to_string())),
			},
		}
	}
}
