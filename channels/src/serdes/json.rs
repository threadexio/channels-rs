use super::prelude::*;

/// The [`mod@serde_json`] serializer which automatically works with all
/// types that implement [`serde::Serialize`] and [`serde::Deserialize`].
///
/// This type is only intended for debugging purposes as it increases
/// packet sizes and only provides human readability. It is meant to aid
/// in understanding the data of the packet in programs like [Wireshark](https://www.wireshark.org/)
/// without having to write a specialized dissector.
#[derive(Debug, Default, Clone)]
pub struct Json();

impl<T> Serializer<T> for Json
where
	T: serde::Serialize,
{
	type Error = serde_json::Error;

	fn serialize<W: Write>(
		&mut self,
		buf: W,
		t: &T,
	) -> Result<(), Self::Error> {
		serde_json::to_writer(buf, t)
	}
}

impl<T> Deserializer<T> for Json
where
	for<'de> T: serde::Deserialize<'de>,
{
	type Error = serde_json::Error;

	fn deserialize<R: Read>(
		&mut self,
		buf: R,
	) -> Result<T, Self::Error> {
		serde_json::from_reader(buf)
	}
}
