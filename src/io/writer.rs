use std::io;

use super::{OwnedBuf, WriteExt};

#[cfg(feature = "statistics")]
use crate::stats;

pub struct Writer<'a> {
	inner: Box<dyn io::Write + 'a>,

	#[cfg(feature = "statistics")]
	stats: stats::SendStats,
}

impl<'a> Writer<'a> {
	pub fn new<W>(writer: W) -> Self
	where
		W: io::Write + 'a,
	{
		Self {
			inner: Box::new(writer),

			#[cfg(feature = "statistics")]
			stats: stats::SendStats::new(),
		}
	}

	pub fn get(&self) -> &dyn io::Write {
		self.inner.as_ref()
	}

	pub fn get_mut(&mut self) -> &mut dyn io::Write {
		self.inner.as_mut()
	}
}

#[cfg(feature = "statistics")]
impl Writer<'_> {
	pub const fn stats(&self) -> &stats::SendStats {
		&self.stats
	}

	pub fn stats_mut(&mut self) -> &mut stats::SendStats {
		&mut self.stats
	}
}

impl io::Write for Writer<'_> {
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

impl WriteExt for Writer<'_> {
	fn write_buffer(
		&mut self,
		buf: &mut OwnedBuf,
		limit: usize,
	) -> io::Result<()> {
		let mut bytes_sent: usize = 0;
		while bytes_sent < limit {
			let remaining = limit - bytes_sent;

			let i = match self.inner.write(&buf.after()[..remaining])
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

			bytes_sent += i;
			buf.seek_forward(i);
		}

		Ok(())
	}
}
