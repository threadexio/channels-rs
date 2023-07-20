use std::io::{Read, Result, Write};

mod cursor;
pub use cursor::*;

#[cfg(feature = "statistics")]
use crate::stats;

pub struct Reader<R> {
	inner: R,

	#[cfg(feature = "statistics")]
	stats: stats::RecvStats,
}

impl<R> Reader<R> {
	pub fn new(reader: R) -> Self {
		Self {
			inner: reader,

			#[cfg(feature = "statistics")]
			stats: stats::RecvStats::new(),
		}
	}

	pub fn get(&self) -> &R {
		&self.inner
	}

	pub fn get_mut(&mut self) -> &mut R {
		&mut self.inner
	}
}

#[cfg(feature = "statistics")]
impl<R> Reader<R> {
	pub fn stats(&self) -> &stats::RecvStats {
		&self.stats
	}

	pub fn stats_mut(&mut self) -> &mut stats::RecvStats {
		&mut self.stats
	}
}

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

pub struct Writer<W> {
	inner: W,

	#[cfg(feature = "statistics")]
	stats: stats::SendStats,
}

impl<W> Writer<W> {
	pub fn new(writer: W) -> Self {
		Self {
			inner: writer,

			#[cfg(feature = "statistics")]
			stats: stats::SendStats::new(),
		}
	}

	pub fn get(&self) -> &W {
		&self.inner
	}

	pub fn get_mut(&mut self) -> &mut W {
		&mut self.inner
	}
}

#[cfg(feature = "statistics")]
impl<W> Writer<W> {
	pub fn stats(&self) -> &stats::SendStats {
		&self.stats
	}

	pub fn stats_mut(&mut self) -> &mut stats::SendStats {
		&mut self.stats
	}
}

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
