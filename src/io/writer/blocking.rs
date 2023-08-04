use super::Writer;

use std::io::{Result, Write};

impl<W> Write for Writer<W>
where
	W: Write,
{
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		let i = self.inner.write(buf)?;

		#[cfg(feature = "statistics")]
		self.stats.add_sent(i);

		Ok(i)
	}

	fn flush(&mut self) -> Result<()> {
		self.inner.flush()
	}
}
