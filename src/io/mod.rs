mod cursor;
pub use cursor::*;

mod reader;
pub use reader::*;

mod writer;
pub use writer::*;

pub use std::io::{Error, ErrorKind, Result};
pub use std::io::{Read, Write};

pub mod prelude {

	pub use super::{BytesMut, BytesRef, Read, Write};
}

/// conditionally call [`ReadExt::fill_buffer`] in order to fill
/// `buf` up to the position `limit`.
pub fn fill_buffer_to(
	buf: &mut OwnedBuf,
	reader: &mut Reader,
	limit: usize,
) -> Result<()> {
	let buf_len = buf.len();
	if buf_len < limit {
		reader.fill_buffer(buf, limit - buf_len)
	} else {
		Ok(())
	}
}
