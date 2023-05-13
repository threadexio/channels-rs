use std::io::Result;

pub trait ReadExt {
	/// Fill the buffer with `limit` new bytes. This method will
	/// continuously call `read()` until `limit` bytes have been
	/// read into `buf` from `self`.
	fn fill_buffer(
		&mut self,
		buf: &mut super::OwnedBuf,
		limit: usize,
	) -> Result<()>;
}

pub trait WriteExt {
	/// Write up to `limit` bytes to `self` from `buf`. This method
	/// will continuously call `write()` until `limit` bytes have been
	/// read into `self` from `buf`.
	fn write_buffer(
		&mut self,
		buf: &mut super::OwnedBuf,
		limit: usize,
	) -> Result<()>;
}
