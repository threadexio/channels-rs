use crate::prelude::*;

pub const MAX_PACKET_SIZE: u16 = u16::MAX;
pub const MAX_PAYLOAD_SIZE: u16 = MAX_PACKET_SIZE - HEADER_SIZE as u16;

pub const PROTOCOL_VERSION: u16 = 0;

pub const HEADER_SIZE: usize = 2 + 2 + 4;

#[derive(Serialize, Deserialize)]
pub struct Header {
	pub protocol_version: u16,
	pub payload_len: u16,

	pub payload_checksum: u32,
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

pub fn serialize<T: Serialize>(data: T) -> Result<Vec<u8>> {
	Ok(bincode!().serialize(&data)?)
}

pub fn serialized_size<T: Serialize>(data: &T) -> Result<u64> {
	Ok(bincode!().serialized_size(data)?)
}

pub fn deserialize<T: DeserializeOwned>(ser_data: &[u8]) -> Result<T> {
	Ok(bincode!().deserialize(ser_data)?)
}
