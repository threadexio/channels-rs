use super::{Reader, Writer};

use std::io::{Read, Result, Write};

impl<R> Read for Reader<R>
where
	R: Read,
{
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let n = self.inner.read(buf)?;

		self.on_read(buf, n)?;
		Ok(n)
	}
}

impl<W> Write for Writer<W>
where
	W: Write,
{
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		let n = self.inner.write(buf)?;

		self.on_write(buf, n)?;
		Ok(n)
	}

	fn flush(&mut self) -> Result<()> {
		self.inner.flush()
	}
}
