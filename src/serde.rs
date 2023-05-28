use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use crate::error::*;

use bincode::Options;
macro_rules! bincode {
	() => {
		bincode::options()
			.reject_trailing_bytes()
			.with_big_endian()
			.with_fixint_encoding()
			.with_no_limit()
	};
}

/// Serialize `t` into `buf` and return the number of bytes
/// written to `buf`.
pub fn serialize<T>(t: &T) -> Result<Vec<u8>>
where
	T: Serialize,
{
	let data = bincode!().serialize(t)?;
	Ok(data)
}

/// Deserialize `buf` into `T`. `buf` must have the exact number
/// of bytes required to encode a `T`.
pub fn deserialize<T>(buf: &[u8]) -> Result<T>
where
	T: DeserializeOwned,
{
	let t = bincode!().deserialize::<T>(buf)?;
	Ok(t)
}
