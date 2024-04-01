//! The [`mod@bincode`] serializer which automatically works with all
//! types that implement [`serde::Serialize`] and [`serde::Deserialize`].

use bincode::Options;

use channels_io::{ContiguousMut, Walkable};

use crate::{Deserializer, Serializer};

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
	#[must_use]
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
	) -> Result<impl Walkable, Self::Error> {
		let vec = default_bincode_config().serialize(t)?;
		Ok(channels_io::Cursor::new(vec))
	}
}

impl<T> Deserializer<T> for Bincode
where
	for<'de> T: serde::Deserialize<'de>,
{
	type Error = bincode::Error;

	fn deserialize(
		&mut self,
		mut buf: impl ContiguousMut,
	) -> Result<T, Self::Error> {
		default_bincode_config().deserialize(buf.chunk_mut())
	}
}
