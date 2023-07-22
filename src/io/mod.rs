use core::any::type_name;
use core::fmt;

use std::io::{Read, Result, Write};

mod cursor;
pub use cursor::*;

mod growable;
pub use growable::*;

mod chain;
pub use chain::Chain;

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

impl<R> fmt::Debug for Reader<R> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_struct("Reader");
		s.field("inner", &type_name::<R>());

		#[cfg(feature = "statistics")]
		s.field("stats", &self.stats);

		s.finish()
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

impl<W> fmt::Debug for Writer<W> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_struct("Writer");
		s.field("inner", &type_name::<W>());

		#[cfg(feature = "statistics")]
		s.field("stats", &self.stats);

		s.finish()
	}
}
