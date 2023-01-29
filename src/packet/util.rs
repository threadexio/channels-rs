use crate::error::*;

#[inline]
pub fn read_offset<T>(buf: &[u8], offset: usize) -> T {
	unsafe { buf.as_ptr().add(offset).cast::<T>().read() }
}

#[inline]
pub fn write_offset<T>(buf: &mut [u8], offset: usize, value: T) {
	unsafe {
		buf.as_mut_ptr().add(offset).cast::<T>().write(value);
	}
}

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

#[inline]
pub fn serialize<T: serde::ser::Serialize>(
	data: &T,
) -> Result<Vec<u8>> {
	Ok(bincode!().serialize(data)?)
}

#[inline]
pub fn deserialize<T: serde::de::DeserializeOwned>(
	data: &[u8],
) -> Result<T> {
	Ok(bincode!().deserialize::<T>(data)?)
}
