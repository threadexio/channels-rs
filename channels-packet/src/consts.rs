pub const HEADER_SIZE_U16: u16 = 8;
pub const HEADER_SIZE_USIZE: usize = HEADER_SIZE_U16 as usize;

/// Protocol Version as seen in the `version` field of the header.
pub const PROTOCOL_VERSION: u16 = 0xfd3f;
