use super::{ErrorKind, OwnedBuf, Result, Write};

#[cfg(feature = "statistics")]
use crate::stats;

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

impl<W> Writer<W>
where
	W: Write,
{
	pub fn write_buffer(
		&mut self,
		buf: &mut OwnedBuf,
		limit: usize,
	) -> Result<()> {
		let mut bytes_sent: usize = 0;
		while bytes_sent < limit {
			let remaining = limit - bytes_sent;

			let i = match self.inner.write(&buf.after()[..remaining])
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

			bytes_sent += i;
			buf.seek_forward(i);
		}

		Ok(())
	}
}
