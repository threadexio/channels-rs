use std::io;

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
	bincode!().serialize(t).map_err(|e| match *e {
		bincode::ErrorKind::SizeLimit => Error::SizeLimit,
		bincode::ErrorKind::Io(io_err)
			if io_err.kind() == io::ErrorKind::WriteZero =>
		{
			Error::SizeLimit
		},
		_ => Error::Serde(e),
	})
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
