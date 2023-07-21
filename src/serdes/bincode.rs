use super::prelude::*;

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

	fn serialize<W: Write>(
		&mut self,
		buf: W,
		t: &T,
	) -> Result<(), Self::Error> {
		bincode!().serialize_into(buf, t)
	}

	fn size_hint(&mut self, t: &T) -> Option<usize> {
		let size_u64 = bincode!().serialized_size(t).ok()?;
		Some(size_u64 as usize)
	}
}

impl<T> Deserializer<T> for Bincode
where
	for<'de> T: serde::Deserialize<'de>,
{
	type Error = bincode::Error;

	fn deserialize<R: Read>(
		&mut self,
		buf: R,
	) -> Result<T, Self::Error> {
		bincode!().deserialize_from(buf)
	}
}
