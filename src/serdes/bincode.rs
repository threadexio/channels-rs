use super::{Deserializer, Serializer};

use bincode::Options;
macro_rules! bincode {
	() => {
		::bincode::options()
			.reject_trailing_bytes()
			.with_big_endian()
			.with_fixint_encoding()
			.with_no_limit()
	};
}

/// The [`mod@bincode`] serializer which automatically works with all
/// types that implement [`serde::Serialize`] and [`serde::Deserialize`].
pub struct Bincode;

impl<T> Serializer<T> for Bincode
where
	T: serde::Serialize,
{
	type Error = bincode::Error;

	fn serialize(&mut self, t: &T) -> Result<Vec<u8>, Self::Error> {
		bincode!().serialize(t)
	}
}

impl<T> Deserializer<T> for Bincode
where
	for<'de> T: serde::Deserialize<'de>,
{
	type Error = bincode::Error;

	fn deserialize(&mut self, buf: &[u8]) -> Result<T, Self::Error> {
		bincode!().deserialize(buf)
	}
}
