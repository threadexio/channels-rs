use super::prelude::*;

use std::io;

/// The [`mod@ciborium`] serializer which automatically works with all
/// types that implement [`serde::Serialize`] and [`serde::Deserialize`].
#[derive(Debug, Default, Clone)]
pub struct Cbor();

impl<T> Serializer<T> for Cbor
where
	T: serde::Serialize,
{
	type Error = ciborium::ser::Error<io::Error>;

	fn serialize<W: Write>(
		&mut self,
		buf: W,
		t: &T,
	) -> Result<(), Self::Error> {
		ciborium::into_writer(t, buf)
	}
}

impl<T> Deserializer<T> for Cbor
where
	for<'de> T: serde::Deserialize<'de>,
{
	type Error = ciborium::de::Error<io::Error>;

	fn deserialize<R: Read>(
		&mut self,
		buf: R,
	) -> Result<T, Self::Error> {
		ciborium::from_reader(buf)
	}
}
