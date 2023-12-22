use alloc::vec::Vec;

use crate::{Deserializer, PayloadBuffer, Serializer};

/// The [`mod@serde_json`] serializer which automatically works with all
/// types that implement [`serde::Serialize`] and [`serde::Deserialize`].
///
/// This type is only intended for debugging purposes as it increases
/// packet sizes and only provides human readability. It is meant to aid
/// in understanding the data of the packet in programs like [Wireshark](https://www.wireshark.org/)
/// without having to write a specialized dissector.
#[derive(Debug, Default, Clone)]
pub struct Json {}

impl Json {
	/// Create a new [`Json`].
	pub fn new() -> Self {
		Self {}
	}
}

impl<T> Serializer<T> for Json
where
	T: serde::Serialize,
{
	type Error = serde_json::Error;

	fn serialize(
		&mut self,
		t: &T,
	) -> Result<PayloadBuffer, Self::Error> {
		let mut buf = PayloadBuffer::new();

		let data = serde_json::to_vec(t)?;
		buf.put_slice(&data);

		Ok(buf)
	}
}

impl<T> Deserializer<T> for Json
where
	for<'de> T: serde::Deserialize<'de>,
{
	type Error = serde_json::Error;

	fn deserialize(
		&mut self,
		buf: &mut Vec<u8>,
	) -> Result<T, Self::Error> {
		serde_json::from_slice(buf)
	}
}
