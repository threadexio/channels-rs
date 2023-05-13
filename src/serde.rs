use core::ops::DerefMut;
use std::io;

use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use crate::error::*;
use crate::io::BorrowedBuf;

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
pub fn serialize<T>(buf: &mut BorrowedBuf, t: &T) -> Result<usize>
where
	T: Serialize,
{
	let old_len = buf.len();
	bincode!().serialize_into(buf.deref_mut(), t).map_err(|e| {
		match *e {
			bincode::ErrorKind::SizeLimit => Error::SizeLimit,
			bincode::ErrorKind::Io(io_err)
				if io_err.kind() == io::ErrorKind::WriteZero =>
			{
				Error::SizeLimit
			},
			_ => Error::Serde(e),
		}
	})?;

	let new_len = buf.len();

	debug_assert!(old_len <= new_len);
	Ok(new_len - old_len)
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
