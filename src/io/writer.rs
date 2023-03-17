#![allow(dead_code)]

use core::ops::{Deref, DerefMut};
use std::io::{self, Write};

use super::{Buffer, WriteExt};

#[cfg(feature = "statistics")]
use crate::stats;

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

impl<W> WriteExt for Writer<W>
where
	W: Write,
{
	fn write_buffer(
		&mut self,
		buf: &mut Buffer,
		limit: usize,
	) -> io::Result<()> {
		let mut bytes_sent: usize = 0;
		while bytes_sent < limit {
			let remaining = limit - bytes_sent;

			let i = match self.write(&buf.after()[..remaining]) {
				Ok(v) if v == 0 => {
					return Err(io::ErrorKind::UnexpectedEof.into())
				},
				Ok(v) => v,
				Err(e) if e.kind() == io::ErrorKind::Interrupted => {
					continue
				},
				Err(e) => return Err(e),
			};

			bytes_sent += i;
			buf.seek_forward(i);
		}

		Ok(())
	}
}
