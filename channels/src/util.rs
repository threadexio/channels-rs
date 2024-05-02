use core::fmt;

use crate::io::{AsyncRead, AsyncWrite, Read, Write};

/// IO statistic information.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Statistics {
	total_bytes: u64,
	packets: u64,
	ops: u64,
}

#[allow(dead_code)]
impl Statistics {
	pub(crate) const fn new() -> Self {
		Self { total_bytes: 0, packets: 0, ops: 0 }
	}

	#[inline]
	pub(crate) fn add_total_bytes(&mut self, n: u64) {
		self.total_bytes += n;
	}

	#[inline]
	pub(crate) fn inc_packets(&mut self) {
		self.packets += 1;
	}

	#[inline]
	pub(crate) fn inc_ops(&mut self) {
		self.ops += 1;
	}
}

#[allow(dead_code)]
impl Statistics {
	/// Returns the number of bytes transferred through this reader/writer.
	#[must_use]
	pub fn total_bytes(&self) -> u64 {
		self.total_bytes
	}

	/// Returns the number of packets transferred through this reader/writer.
	#[must_use]
	pub fn packets(&self) -> u64 {
		self.packets
	}

	/// Returns the total number of `send`/`recv` operations.
	#[must_use]
	pub fn ops(&self) -> u64 {
		self.ops
	}
}

impl fmt::Debug for Statistics {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Statistics")
			.field("total_bytes", &self.total_bytes)
			.field("packets", &self.packets)
			.field("ops", &self.ops)
			.finish()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StatIO<R> {
	pub(crate) inner: R,

	#[cfg(feature = "statistics")]
	pub(crate) statistics: Statistics,
}

#[allow(unused_variables, clippy::unused_self)]
impl<R> StatIO<R> {
	pub fn new(reader: R) -> Self {
		Self {
			inner: reader,

			#[cfg(feature = "statistics")]
			statistics: Statistics::new(),
		}
	}

	fn on_read(&mut self, n: u64) {
		#[cfg(feature = "statistics")]
		self.statistics.add_total_bytes(n);
	}

	fn on_write(&mut self, n: u64) {
		#[cfg(feature = "statistics")]
		self.statistics.add_total_bytes(n);
	}
}

impl<W: Write> Write for StatIO<W> {
	type Error = W::Error;

	fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
		self.inner.write(buf)?;

		let dl = buf.len();
		self.on_write(dl as u64);
		Ok(())
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
		self.inner.flush()
	}
}

impl<W: AsyncWrite> AsyncWrite for StatIO<W> {
	type Error = W::Error;

	async fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
		self.inner.write(buf).await?;

		let dl = buf.len();
		self.on_write(dl as u64);
		Ok(())
	}

	async fn flush(&mut self) -> Result<(), Self::Error> {
		self.inner.flush().await
	}
}

impl<R: Read> Read for StatIO<R> {
	type Error = R::Error;

	fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
		self.inner.read(buf)?;

		let dl = buf.len();
		self.on_read(dl as u64);
		Ok(())
	}
}

impl<R: AsyncRead> AsyncRead for StatIO<R> {
	type Error = R::Error;

	async fn read(
		&mut self,
		buf: &mut [u8],
	) -> Result<(), Self::Error> {
		self.inner.read(buf).await?;

		let dl = buf.len();
		self.on_read(dl as u64);
		Ok(())
	}
}
