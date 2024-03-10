use crate::{Contiguous, Deserializer, Serializer, Walkable};

/// The [`mod@ciborium`] serializer which automatically works with all
/// types that implement [`serde::Serialize`] and [`serde::Deserialize`].
#[derive(Debug, Default, Clone)]
pub struct Cbor {}

impl Cbor {
	/// Create a new [`Cbor`].
	#[must_use]
	pub fn new() -> Self {
		Self {}
	}
}

impl<T> Serializer<T> for Cbor
where
	T: serde::Serialize,
{
	type Error = ciborium::ser::Error<std::io::Error>;

	fn serialize(
		&mut self,
		t: &T,
	) -> Result<impl Walkable, Self::Error> {
		let mut buf = Vec::new();
		ciborium::into_writer(t, &mut buf)?;

		Ok(channels_io::Cursor::new(buf))
	}
}

impl<T> Deserializer<T> for Cbor
where
	for<'de> T: serde::Deserialize<'de>,
{
	type Error = ciborium::de::Error<std::io::Error>;

	fn deserialize(
		&mut self,
		buf: impl Contiguous,
	) -> Result<T, Self::Error> {
		ciborium::from_reader(buf.chunk())
	}
}
