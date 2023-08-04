use super::Reader;

use std::io::{Read, Result};

impl<R> Read for Reader<R>
where
	R: Read,
{
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let i = self.inner.read(buf)?;

		#[cfg(feature = "statistics")]
		self.stats.add_received(i);

		Ok(i)
	}
}
