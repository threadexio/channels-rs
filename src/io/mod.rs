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
