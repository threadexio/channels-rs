use super::{OwnedBuf, Read, Result, Write};

pub trait ReadExt: Read {
	/// Fill the buffer with `limit` new bytes. This method will
	/// continuously call `read()` until `limit` bytes have been
	/// read into `buf` from `self`.
	fn fill_buffer(
		&mut self,
		buf: &mut OwnedBuf,
		limit: usize,
	) -> Result<()>;
}

impl<T> ReadExt for &mut T
where
	T: ReadExt,
{
	fn fill_buffer(
		&mut self,
		buf: &mut OwnedBuf,
		limit: usize,
	) -> Result<()> {
		(**self).fill_buffer(buf, limit)
	}
}

pub trait WriteExt: Write {
	/// Write up to `limit` bytes to `self` from `buf`. This method
	/// will continuously call `write()` until `limit` bytes have been
	/// read into `self` from `buf`.
	fn write_buffer(
		&mut self,
		buf: &mut OwnedBuf,
		limit: usize,
	) -> Result<()>;
}

impl<T> WriteExt for &mut T
where
	T: WriteExt,
{
	fn write_buffer(
		&mut self,
		buf: &mut OwnedBuf,
		limit: usize,
	) -> Result<()> {
		(**self).write_buffer(buf, limit)
	}
}
