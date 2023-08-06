use super::Writer;

use std::io::{Result, Write};

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
