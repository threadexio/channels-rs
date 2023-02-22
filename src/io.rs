#![allow(dead_code)]
use std::io::{self, Read, Write};
use std::ops::{Deref, DerefMut};

#[cfg(feature = "statistics")]
use crate::stats;

pub struct Reader<R> {
	inner: R,

	#[cfg(feature = "statistics")]
	stats: stats::RecvStats,
}

impl<R> Reader<R>
where
	R: Read,
{
	pub fn new(reader: R) -> Self {
		Self {
			inner: reader,

			#[cfg(feature = "statistics")]
			stats: stats::RecvStats::new(),
		}
	}

	pub fn into_inner(self) -> R {
		self.inner
	}

	#[cfg(feature = "statistics")]
	pub fn stats(&self) -> &stats::RecvStats {
		&self.stats
	}

	#[cfg(feature = "statistics")]
	pub fn stats_mut(&mut self) -> &mut stats::RecvStats {
		&mut self.stats
	}
}

impl<R> Read for Reader<R>
where
	R: Read,
{
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let i = self.inner.read(buf)?;

		#[cfg(feature = "statistics")]
		self.stats.add_received(i);

		Ok(i)
	}
}

impl<R> Deref for Reader<R> {
	type Target = R;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<R> DerefMut for Reader<R> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

pub struct Writer<W> {
	inner: W,

	#[cfg(feature = "statistics")]
	stats: stats::SendStats,
}

impl<W> Writer<W>
where
	W: Write,
{
	pub fn new(writer: W) -> Self {
		Self {
			inner: writer,

			#[cfg(feature = "statistics")]
			stats: stats::SendStats::new(),
		}
	}

	pub fn into_inner(self) -> W {
		self.inner
	}

	#[cfg(feature = "statistics")]
	pub fn stats(&self) -> &stats::SendStats {
		&self.stats
	}

	#[cfg(feature = "statistics")]
	pub fn stats_mut(&mut self) -> &mut stats::SendStats {
		&mut self.stats
	}
}

impl<W> Write for Writer<W>
where
	W: Write,
{
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		let i = self.inner.write(buf)?;

		#[cfg(feature = "statistics")]
		self.stats.add_sent(i);

		Ok(i)
	}

	fn flush(&mut self) -> io::Result<()> {
		self.inner.flush()
	}
}

impl<W> Deref for Writer<W> {
	type Target = W;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<W> DerefMut for Writer<W> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}
