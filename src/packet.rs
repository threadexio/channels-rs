use crate::prelude::*;

pub type Length = u16;

pub const HEADER_SIZE: usize = std::mem::size_of::<Header>();

pub const MAX_PACKET_SIZE: Length = 0xffff;
pub const MAX_MESSAGE_SIZE: Length = MAX_PACKET_SIZE - HEADER_SIZE as Length;

pub fn serialize<T: Serialize>(data: T) -> Result<Vec<u8>> {
	bincode::options()
		.reject_trailing_bytes()
		.with_big_endian()
		.with_fixint_encoding()
		.with_limit(crate::packet::MAX_MESSAGE_SIZE as u64)
		.serialize(&data)
		.map_err(|x| other!("{}", x))
}

pub fn deserialize<T: DeserializeOwned>(ser_data: &[u8]) -> Result<T> {
	bincode::options()
		.reject_trailing_bytes()
		.with_big_endian()
		.with_fixint_encoding()
		.with_limit(crate::packet::MAX_MESSAGE_SIZE as u64)
		.deserialize(ser_data)
		.map_err(|x| other!("{}", x))
}

#[derive(Serialize, Deserialize)]
pub struct Header {
	pub payload_len: Length,
}
