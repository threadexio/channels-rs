use core::convert::Infallible;

use alloc::vec;
use alloc::vec::Vec;

use channels::serdes::{Crc, Deserializer, Serializer};

use crate::record::{Record, RecordPath, RecordType};

pub type Serdes = Crc<RecordSerdes>;

#[derive(Debug, Default, Clone, Copy)]
pub struct RecordSerdes;

const RECORD_SIZE: usize =
	8 /* timestamp */
	+ 2 /* typ */
	+ 16 /* path */;

impl Serializer<Record> for RecordSerdes {
	type Error = Infallible;

	fn serialize(
		&mut self,
		record: &Record,
	) -> Result<Vec<u8>, Self::Error> {
		let mut buf = vec![0u8; RECORD_SIZE];

		let Record { timestamp, typ, path } = record;

		let timestamp_bytes = timestamp.to_be_bytes();
		let typ_bytes = match typ {
			RecordType::Get => 1u16,
			RecordType::Update => 2u16,
		}
		.to_ne_bytes();

		buf[0..8].copy_from_slice(&timestamp_bytes[..]);
		buf[8..10].copy_from_slice(&typ_bytes[..]);
		buf[10..26].copy_from_slice(&path.0[..]);

		Ok(buf)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordDeserializeError {
	NotEnough,
	InvalidType,
}

impl Deserializer<Record> for RecordSerdes {
	type Error = RecordDeserializeError;

	fn deserialize(
		&mut self,
		buf: &mut [u8],
	) -> Result<Record, Self::Error> {
		if buf.len() < RECORD_SIZE {
			return Err(RecordDeserializeError::NotEnough);
		}

		let timestamp =
			u64::from_be_bytes(buf[0..8].try_into().unwrap());

		let typ = match u16::from_be_bytes(
			buf[8..10].try_into().unwrap(),
		) {
			1 => RecordType::Get,
			2 => RecordType::Update,
			_ => return Err(RecordDeserializeError::InvalidType),
		};

		let path = RecordPath(buf[10..26].try_into().unwrap());

		Ok(Record { timestamp, typ, path })
	}
}
