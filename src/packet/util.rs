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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_read_offset() {
		let buf = &[0u8, 1, 2, 3];

		assert_eq!(read_offset::<u8>(buf, 0), 0);
		assert_eq!(read_offset::<u8>(buf, 1), 1);
		assert_eq!(read_offset::<u8>(buf, 2), 2);
		assert_eq!(read_offset::<u8>(buf, 3), 3);

		assert_eq!(
			read_offset::<u16>(buf, 0),
			u16::from_ne_bytes([0, 1])
		);

		assert_eq!(
			read_offset::<u16>(buf, 1),
			u16::from_ne_bytes([1, 2])
		);

		assert_eq!(
			read_offset::<u16>(buf, 2),
			u16::from_ne_bytes([2, 3])
		);

		assert_eq!(
			read_offset::<u32>(buf, 0),
			u32::from_ne_bytes([0, 1, 2, 3])
		);
	}

	#[test]
	fn test_write_offset() {
		let buf = &mut [0u8, 1, 2, 3];

		write_offset::<u8>(buf, 0, 0xa1);
		write_offset::<u8>(buf, 1, 0xa2);
		write_offset::<u8>(buf, 2, 0xa3);
		write_offset::<u8>(buf, 3, 0xa4);
		assert_eq!(buf, &[0xa1, 0xa2, 0xa3, 0xa4]);

		write_offset::<u16>(buf, 0, u16::from_ne_bytes([0xb1, 0xb2]));
		assert_eq!(buf, &[0xb1, 0xb2, 0xa3, 0xa4]);

		write_offset::<u16>(buf, 1, u16::from_ne_bytes([0xb3, 0xb4]));
		assert_eq!(buf, &[0xb1, 0xb3, 0xb4, 0xa4]);

		write_offset::<u16>(buf, 2, u16::from_ne_bytes([0xb6, 0xb7]));
		assert_eq!(buf, &[0xb1, 0xb3, 0xb6, 0xb7]);

		write_offset::<u32>(
			buf,
			0,
			u32::from_ne_bytes([0xc1, 0xc2, 0xc3, 0xc4]),
		);
		assert_eq!(buf, &[0xc1, 0xc2, 0xc3, 0xc4]);
	}
}
