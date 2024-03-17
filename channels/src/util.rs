use core::fmt;

use channels_packet::IdGenerator;

use crate::io::{
	AsyncRead, AsyncWrite, Contiguous, ContiguousMut, Read, Write,
};

#[derive(Clone)]
pub struct Pcb {
	pub id_gen: IdGenerator,
}

impl Pcb {
	pub const fn new() -> Self {
		Self { id_gen: IdGenerator::new() }
	}
}

/// IO statistic information.
#[derive(Clone)]
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

#[derive(Debug, Clone)]
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

	fn write<B>(&mut self, mut buf: B) -> Result<(), Self::Error>
	where
		B: Contiguous,
	{
		let l0 = buf.remaining();
		self.inner.write(&mut buf)?;
		let l1 = buf.remaining();

		let dl = usize::abs_diff(l0, l1);
		self.on_write(dl as u64);
		Ok(())
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
		self.inner.flush()
	}
}

impl<W: AsyncWrite> AsyncWrite for StatIO<W> {
	type Error = W::Error;

	async fn write<B>(
		&mut self,
		mut buf: B,
	) -> Result<(), Self::Error>
	where
		B: Contiguous,
	{
		let l0 = buf.remaining();
		self.inner.write(&mut buf).await?;
		let l1 = buf.remaining();

		let dl = usize::abs_diff(l0, l1);
		self.on_write(dl as u64);
		Ok(())
	}

	async fn flush(&mut self) -> Result<(), Self::Error> {
		self.inner.flush().await
	}
}

impl<R: Read> Read for StatIO<R> {
	type Error = R::Error;

	fn read<B>(&mut self, mut buf: B) -> Result<(), Self::Error>
	where
		B: ContiguousMut,
	{
		let l0 = buf.remaining_mut();
		self.inner.read(&mut buf)?;
		let l1 = buf.remaining_mut();

		let dl = usize::abs_diff(l0, l1);
		self.on_read(dl as u64);
		Ok(())
	}
}

impl<R: AsyncRead> AsyncRead for StatIO<R> {
	type Error = R::Error;

	async fn read<B>(&mut self, mut buf: B) -> Result<(), Self::Error>
	where
		B: ContiguousMut,
	{
		let l0 = buf.remaining_mut();
		self.inner.read(&mut buf).await?;
		let l1 = buf.remaining_mut();

		let dl = usize::abs_diff(l0, l1);
		self.on_read(dl as u64);
		Ok(())
	}
}
