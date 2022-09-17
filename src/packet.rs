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

pub fn serialize<T: Serialize>(data: &T) -> Result<Vec<u8>> {
	Ok(bincode!().serialize(&data)?)
}

pub fn serialized_size<T: Serialize>(data: &T) -> Result<u64> {
	Ok(bincode!().serialized_size(data)?)
}

pub fn deserialize<T: DeserializeOwned>(ser_data: &[u8]) -> Result<T> {
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
	($b:expr, $offset:expr, $type:ty, $fn:ident) => {
		<$type>::$fn(read_offset!($b, $offset, $type))
	};
}

macro_rules! write_offset {
	($b:expr, $offset:expr, $value:expr, $type:ty) => {
		unsafe {
			*($b.as_mut_ptr().add($offset).cast::<$type>()) = $value;
		}
	};
	($b:expr, $offset:expr, $value:expr, $type:ty, $fn:ident) => {
		write_offset!($b, $offset, <$type>::$fn($value), $type)
	};
}

macro_rules! header {
	(
		// field offset          field name         field type
		//
		$($field_name:ident: $field_type:ty,)*
	) => {
		pub struct Header<'a> {
			pub buffer: &'a mut [u8],
		}

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
		}
	};
}

pub const MAX_PACKET_SIZE: u16 = u16::MAX;
pub const MAX_PAYLOAD_SIZE: u16 = MAX_PACKET_SIZE - Header::SIZE as u16;

pub const PROTOCOL_VERSION: u16 = 0;

header! {
	protocol_version:	u16,
	header_checksum:	u16,
	payload_len:		u16,
	payload_checksum:	u16,
}

#[allow(dead_code)]
impl Header<'_> {
	pub fn protocol_version(&self) -> u16 {
		read_offset!(self.buffer, 0, u16, from_be)
	}

	pub fn header_checksum(&self) -> u16 {
		read_offset!(self.buffer, 2, u16, from_be)
	}

	pub fn payload_len(&self) -> u16 {
		read_offset!(self.buffer, 4, u16, from_be)
	}

	pub fn payload_checksum(&self) -> u16 {
		read_offset!(self.buffer, 6, u16, from_be)
	}

	pub fn set_protocol_version(&mut self, new: u16) -> &[u8] {
		write_offset!(self.buffer, 0, new, u16, to_be);
		&self.buffer[0..2]
	}

	pub fn set_header_checksum(&mut self, new: u16) -> &[u8] {
		write_offset!(self.buffer, 2, new, u16, to_be);
		&self.buffer[2..4]
	}

	pub fn set_payload_len(&mut self, new: u16) -> &[u8] {
		write_offset!(self.buffer, 4, new, u16, to_be);
		&self.buffer[4..6]
	}

	pub fn set_payload_checksum(&mut self, new: u16) -> &[u8] {
		write_offset!(self.buffer, 6, new, u16, to_be);
		&self.buffer[6..8]
	}
}
