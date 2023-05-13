use std::io::Result;

pub trait ReadExt {
	fn fill_buffer(
		&mut self,
		buf: &mut super::OwnedBuf,
		limit: usize,
	) -> Result<()>;
}

pub trait WriteExt {
	fn write_buffer(
		&mut self,
		buf: &mut super::OwnedBuf,
		limit: usize,
	) -> Result<()>;
}
