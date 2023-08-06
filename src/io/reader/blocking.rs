use super::Reader;

use std::io::{Read, Result};

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
