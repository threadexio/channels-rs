use super::{ErrorKind, OwnedBuf, Read, Result};

#[cfg(feature = "statistics")]
use crate::stats;

pub struct Reader<R>
where
	R: Read,
{
	inner: R,

	#[cfg(feature = "statistics")]
	stats: stats::RecvStats,
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

	pub fn get(&self) -> &R {
		&self.inner
	}

	pub fn get_mut(&mut self) -> &mut R {
		&mut self.inner
	}

	pub fn fill_buffer(
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

#[cfg(feature = "statistics")]
impl<R> Reader<R>
where
	R: Read,
{
	pub fn stats(&self) -> &stats::RecvStats {
		&self.stats
	}

	pub fn stats_mut(&mut self) -> &mut stats::RecvStats {
		&mut self.stats
	}
}
