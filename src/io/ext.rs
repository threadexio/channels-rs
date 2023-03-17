use std::io::Result;

use super::Buffer;

pub trait ReadExt {
	fn fill_buffer(
		&mut self,
		buf: &mut Buffer,
		limit: usize,
	) -> Result<()>;
}

pub trait WriteExt {
	fn write_buffer(
		&mut self,
		buf: &mut Buffer,
		limit: usize,
	) -> Result<()>;
}
