use core::convert::Infallible;

use channels::{
	io::{Contiguous, Cursor, Walkable},
	serdes::{Crc, Deserializer, Serializer},
};

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
	) -> Result<impl Walkable, Self::Error> {
		let mut buf = [0u8; RECORD_SIZE];

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

		Ok(Cursor::new(buf))
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
		buf: impl Contiguous,
	) -> Result<Record, Self::Error> {
		let buf = buf.chunk();

		if buf.len() < RECORD_SIZE {
			return Err(RecordDeserializeError::NotEnough);
		}

		let mut timestamp_bytes = [0u8; 8];
		timestamp_bytes[..].copy_from_slice(&buf[0..8]);
		let timestamp = u64::from_be_bytes(timestamp_bytes);

		let mut typ_bytes = [0u8; 2];
		typ_bytes[..].copy_from_slice(&buf[8..10]);
		let typ = match u16::from_be_bytes(typ_bytes) {
			1 => RecordType::Get,
			2 => RecordType::Update,
			_ => return Err(RecordDeserializeError::InvalidType),
		};

		let mut path_bytes = [0u8; 16];
		path_bytes[..].copy_from_slice(&buf[10..26]);
		let path = RecordPath(path_bytes);

		Ok(Record { timestamp, typ, path })
	}
}
