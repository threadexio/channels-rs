use crate::prelude::*;

macro_rules! define_struct {
	($struct_vis:vis $name:ident {
		$($vis:vis $field_name:ident: $field_type:ty = $default:expr,)*
	}) => {
		#[derive(Debug, Serialize, Deserialize)]
		$struct_vis struct $name {
			$($vis $field_name: $field_type,)*
		}

		impl $name {
			pub const SIZE: usize = 0 $( + std::mem::size_of::<$field_type>())*;
		}

		impl Default for $name {
			fn default() -> Self {
				Self {
					$($field_name: $default,)*
				}
			}
		}
	};
}

macro_rules! bincode {
	() => {
		bincode::options()
			.reject_trailing_bytes()
			.with_big_endian()
			.with_fixint_encoding()
			.with_no_limit()
	};
}

pub const MAX_PACKET_SIZE: u16 = u16::MAX;
pub const MAX_PAYLOAD_SIZE: u16 = MAX_PACKET_SIZE - Header::SIZE as u16;

pub const PROTOCOL_VERSION: u16 = 0;

define_struct! {
	pub Header {
		pub protocol_version: u16 = PROTOCOL_VERSION,
		pub payload_len: u16 = 0,
		pub payload_checksum: u16 = 0,
	}
}

pub fn serialize<T: Serialize>(data: T) -> Result<Vec<u8>> {
	Ok(bincode!().serialize(&data)?)
}

pub fn serialized_size<T: Serialize>(data: &T) -> Result<u64> {
	Ok(bincode!().serialized_size(data)?)
}

pub fn deserialize<T: DeserializeOwned>(ser_data: &[u8]) -> Result<T> {
	Ok(bincode!().deserialize(ser_data)?)
}
