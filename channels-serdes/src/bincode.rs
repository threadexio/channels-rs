use alloc::vec::Vec;

use bincode::Options;

use super::{Deserializer, Serializer};

fn default_bincode_config() -> impl Options {
	bincode::options()
		.reject_trailing_bytes()
		.with_big_endian()
		.with_fixint_encoding()
		.with_no_limit()
}

/// The [`mod@bincode`] serializer which automatically works with all
/// types that implement [`serde::Serialize`] and [`serde::Deserialize`].
#[derive(Debug, Default, Clone)]
pub struct Bincode {}

impl Bincode {
	/// Create a new [`Bincode`] with the default options.
	pub fn new() -> Self {
		Self {}
	}

	// TODO: Constructor that accepts a clojure that creates the bincode config
	//
	// Possible implementation:
	/*
	pub fn with_config<F>(f: F) -> Self
	where
		F: Fn() -> impl Options,
	{            // ^ coming in 1.74.0 (hopefully)
		Self { config: f }
	}
	*/
}

impl<T> Serializer<T> for Bincode
where
	T: serde::Serialize,
{
	type Error = bincode::Error;

	fn serialize(&mut self, t: &T) -> Result<Vec<u8>, Self::Error> {
		default_bincode_config().serialize(t)
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
