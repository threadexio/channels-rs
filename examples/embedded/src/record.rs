#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecordType {
	Get,
	Update,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordPath(pub [u8; 16]);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Record {
	pub timestamp: u64,
	pub typ: RecordType,
	pub path: RecordPath,
}
