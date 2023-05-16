use super::{ErrorKind, OwnedBuf, Read, ReadExt, Result};

#[cfg(feature = "statistics")]
use crate::stats;

pub struct Reader<'a> {
	inner: Box<dyn Read + 'a>,

	#[cfg(feature = "statistics")]
	stats: stats::RecvStats,
}

impl<'a> Reader<'a> {
	pub fn new<R>(reader: R) -> Self
	where
		R: Read + 'a,
	{
		Self {
			inner: Box::new(reader),

			#[cfg(feature = "statistics")]
			stats: stats::RecvStats::new(),
		}
	}

	pub fn get(&self) -> &dyn Read {
		self.inner.as_ref()
	}

	pub fn get_mut(&mut self) -> &mut dyn Read {
		self.inner.as_mut()
	}
}

#[cfg(feature = "statistics")]
impl Reader<'_> {
	pub fn stats(&self) -> &stats::RecvStats {
		&self.stats
	}

	pub fn stats_mut(&mut self) -> &mut stats::RecvStats {
		&mut self.stats
	}
}

impl Read for Reader<'_> {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let i = self.inner.read(buf)?;

		#[cfg(feature = "statistics")]
		self.stats.add_received(i);

		Ok(i)
	}
}

impl ReadExt for Reader<'_> {
	fn fill_buffer(
		&mut self,
		buf: &mut OwnedBuf,
		limit: usize,
	) -> Result<()> {
		let mut bytes_read: usize = 0;
		while limit > bytes_read {
			let remaining = limit - bytes_read;

			let i = match self
				.inner
				.read(&mut buf.after_mut()[..remaining])
			{
				Ok(v) if v == 0 => {
					return Err(ErrorKind::UnexpectedEof.into())
				},
				Ok(v) => v,
				Err(e) if e.kind() == ErrorKind::Interrupted => {
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
