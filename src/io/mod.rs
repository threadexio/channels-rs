mod cursor;
pub use cursor::*;

mod ext;
pub use ext::*;

mod reader;
pub use reader::*;

mod writer;
pub use writer::*;

pub use std::io::{Error, ErrorKind, Result};
pub use std::io::{Read, Write};

pub mod prelude {

	pub use super::{
		BytesMut, BytesRef, Read, ReadExt, Write, WriteExt,
	};
}

/// conditionally call [`ReadExt::fill_buffer`] in order to fill
/// `buf` up to the position `limit`.
pub fn fill_buffer_to<R>(
	buf: &mut OwnedBuf,
	mut reader: R,
	limit: usize,
) -> Result<()>
where
	R: ReadExt,
{
	let buf_len = buf.len();
	if buf_len < limit {
		reader.fill_buffer(buf, limit - buf_len)
	} else {
		Ok(())
	}
}
