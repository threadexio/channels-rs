use core::ops::{Deref, DerefMut};
use std::io::{self, Read};

use super::{Buffer, ReadExt};

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

impl<R> ReadExt for Reader<R>
where
	R: Read,
{
	fn fill_buffer(
		&mut self,
		buf: &mut Buffer,
		limit: usize,
	) -> io::Result<()> {
		let mut bytes_read: usize = 0;
		while limit > bytes_read {
			let remaining = limit - bytes_read;

			let i = match self.read(&mut buf.after_mut()[..remaining])
			{
				Ok(v) if v == 0 => {
					return Err(io::ErrorKind::UnexpectedEof.into())
				},
				Ok(v) => v,
				Err(e) if e.kind() == io::ErrorKind::Interrupted => {
					continue
				},
				Err(e) => return Err(e),
			};

			buf.seek_forward(i);
			bytes_read += i;
		}

		Ok(())
	}
}
