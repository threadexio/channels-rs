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

pub fn serialize<T: serde::ser::Serialize>(
	data: &T,
) -> Result<Vec<u8>> {
	Ok(bincode!().serialize(data)?)
}

pub fn deserialize<T: serde::de::DeserializeOwned>(
	data: &[u8],
) -> Result<T> {
	Ok(bincode!().deserialize::<T>(data)?)
}
