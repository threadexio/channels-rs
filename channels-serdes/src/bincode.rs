use alloc::vec::Vec;

use bincode::Options;

use crate::{Deserializer, PayloadBuffer, Serializer};

fn default_bincode_config() -> impl Options {
	bincode::options()
		.reject_trailing_bytes()
		.with_big_endian()
		.with_fixint_encoding()
		.with_no_limit()
}

/// The [`mod@bincode`] serializer which automatically works with all
/// types that implement [`serde::Serialize`] and [`serde::Deserialize`].
///
/// Note that the default options this crate uses are not the same as the
/// ones from [`bincode::options()`].
///
/// Default configuration:
///
/// - **Byte Limit**: `Unlimited`
/// - **Endianness**: `Big`
/// - **Int Encoding**: `Fixint`
/// - **Trailing Behavior**: `Reject`
///
/// **NOTE:** If you want to use other options with [`mod@bincode`] you must
/// implement your own serializer and deserializer.
#[derive(Debug, Default, Clone)]
pub struct Bincode {}

impl Bincode {
	/// Create a new [`Bincode`] with the default options.
	pub fn new() -> Self {
		Self {}
	}
}

impl<T> Serializer<T> for Bincode
where
	T: serde::Serialize,
{
	type Error = bincode::Error;

	fn serialize(
		&mut self,

		t: &T,
	) -> Result<PayloadBuffer, Self::Error> {
		let mut buf = PayloadBuffer::new();
		let bincode = &mut default_bincode_config();

		let size_hint = bincode.serialized_size(t)?;
		if let Ok(size_hint) = usize::try_from(size_hint) {
			buf.reserve(size_hint);
		}

		bincode.serialize_into(&mut buf, t).map(|_| buf)
	}
}

impl<T> Deserializer<T> for Bincode
where
	for<'de> T: serde::Deserialize<'de>,
{
	type Error = bincode::Error;

	fn deserialize(
		&mut self,
		buf: &mut Vec<u8>,
	) -> Result<T, Self::Error> {
		default_bincode_config().deserialize(buf)
	}
}
