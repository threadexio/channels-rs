use crate::prelude::*;

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
pub fn serialize<T: Serialize>(data: &T) -> Result<Vec<u8>> {
	Ok(bincode!().serialize(&data)?)
}

#[inline]
pub fn serialized_size<T: Serialize>(data: &T) -> Result<u64> {
	Ok(bincode!().serialized_size(data)?)
}

#[inline]
pub fn deserialize<T: DeserializeOwned>(
	ser_data: &[u8],
) -> Result<T> {
	Ok(bincode!().deserialize(ser_data)?)
}

macro_rules! sizeof {
	($t:ty) => {
		(std::mem::size_of::<$t>())
	};
}

macro_rules! read_offset {
	($b:expr, $offset:expr, $type:ty) => {
		unsafe { *($b.as_ptr().add($offset).cast::<$type>()) }
	};
	($b:expr, $offset:expr, $type:ty, $fn:expr) => {
		$fn(read_offset!($b, $offset, $type))
	};
}

macro_rules! write_offset {
	($b:expr, $offset:expr, $value:expr, $type:ty) => {
		unsafe {
			*($b.as_mut_ptr().add($offset).cast::<$type>()) = $value;
		}
	};
	($b:expr, $offset:expr, $value:expr, $type:ty, $fn:expr) => {
		write_offset!($b, $offset, $fn($value), $type)
	};
}

macro_rules! header {
	(
		// field offset          field name         field type
		//
		$($field_offset:expr => $field_name:ident: $field_type:ty { s $(= $ser_fn:expr)?, d $(= $de_fn:expr)? },)*
	) => {
		pub struct Header<'a> {
			pub buffer: &'a mut [u8],
		}

		#[allow(dead_code)]
		impl<'a> Header<'a> {
			pub const SIZE: usize = 0 $( + sizeof!($field_type))*;

			pub fn new(buffer: &'a mut [u8]) -> Self {
				debug_assert!(buffer.len() >= Self::SIZE);

				Self {
					buffer,
				}
			}

			pub fn get(&self) -> &[u8] {
				&self.buffer
			}

			$(
				concat_idents::concat_idents!(getter_name = get_, $field_name {
					pub fn getter_name(&self) -> $field_type {
						read_offset!(self.buffer, $field_offset, $field_type $(, $de_fn)?)
					}
				});

				concat_idents::concat_idents!(setter_name = set_, $field_name {
					pub fn setter_name(&mut self, x: $field_type) -> &[u8] {
						write_offset!(self.buffer, $field_offset, x, $field_type $(, $ser_fn)?);
						&self.buffer[$field_offset..($field_offset + sizeof!($field_type))]
					}
				});

			)*
		}
	};
}

pub const MAX_PACKET_SIZE: u16 = u16::MAX;
pub const MAX_PAYLOAD_SIZE: u16 =
	MAX_PACKET_SIZE - Header::SIZE as u16;

pub const PROTOCOL_VERSION: u16 = 0;

header! {
	0 => protocol_version:	u16 {s = u16::to_be, d = u16::from_be},
	2 => header_checksum:	u16 {s = u16::to_be, d = u16::from_be},
	4 => payload_len:		u16 {s = u16::to_be, d = u16::from_be},
	6 => payload_checksum:	u16 {s = u16::to_be, d = u16::from_be},
}
