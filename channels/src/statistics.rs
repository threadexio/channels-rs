use core::fmt;
use core::hash::Hash;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use pin_project::pin_project;

use crate::io::{AsyncRead, AsyncWrite, Read, Write};

trait Collector: Default + Clone + PartialEq + Eq + Hash {
	fn total_bytes(&self) -> u64;
	fn add_total_bytes(&mut self, n: u64);

	fn total_items(&self) -> u64;
	fn inc_total_items(&mut self);

	fn io_ops(&self) -> u64;
	fn inc_ops(&mut self);
}

#[allow(unused)]
#[derive(Clone, Default, PartialEq, Eq, Hash)]
struct RealStatistics {
	total_bytes: u64,
	total_items: u64,
	io_ops: u64,
}

impl Collector for RealStatistics {
	fn total_bytes(&self) -> u64 {
		self.total_bytes
	}

	fn add_total_bytes(&mut self, n: u64) {
		self.total_bytes += n;
	}

	fn total_items(&self) -> u64 {
		self.total_items
	}

	fn inc_total_items(&mut self) {
		self.total_items += 1;
	}

	fn io_ops(&self) -> u64 {
		self.io_ops
	}

	fn inc_ops(&mut self) {
		self.io_ops += 1;
	}
}

#[allow(unused)]
#[derive(Clone, Default, PartialEq, Eq, Hash)]
struct MockStatistics;

impl Collector for MockStatistics {
	fn total_bytes(&self) -> u64 {
		0
	}

	fn add_total_bytes(&mut self, _: u64) {}

	fn total_items(&self) -> u64 {
		0
	}

	fn inc_total_items(&mut self) {}

	fn io_ops(&self) -> u64 {
		0
	}

	fn inc_ops(&mut self) {}
}

#[cfg(feature = "statistics")]
use self::RealStatistics as StatisticsImpl;

#[cfg(not(feature = "statistics"))]
use self::MockStatistics as StatisticsImpl;

/// IO statistic information.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Statistics(StatisticsImpl);

impl Statistics {
	pub(crate) fn new() -> Self {
		Self(StatisticsImpl::default())
	}

	#[inline]
	fn add_total_bytes(&mut self, n: u64) {
		self.0.add_total_bytes(n);
	}

	#[inline]
	pub(crate) fn inc_total_items(&mut self) {
		self.0.inc_total_items();
	}

	#[inline]
	fn inc_ops(&mut self) {
		self.0.inc_ops();
	}

	/// Returns the number of bytes transferred through this reader/writer.
	#[inline]
	#[must_use]
	pub fn total_bytes(&self) -> u64 {
		self.0.total_bytes()
	}

	/// Returns the total number of items transferred though this reader/writer.
	///
	/// An item is a logical boundary that is marked by the framing layer. Essentially,
	/// an item is one call to `send`/`recv`.
	#[inline]
	#[must_use]
	pub fn total_items(&self) -> u64 {
		self.0.total_items()
	}

	/// Returns the total number of IO operations.
	#[inline]
	#[must_use]
	pub fn io_ops(&self) -> u64 {
		self.0.io_ops()
	}
}

impl fmt::Debug for Statistics {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Statistics")
			.field("total_bytes", &self.total_bytes())
			.field("total_items", &self.total_items())
			.field("io_ops", &self.io_ops())
			.finish()
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
#[pin_project]
pub struct StatIO<R> {
	#[pin]
	pub(crate) inner: R,
	pub(crate) statistics: Statistics,
}

#[allow(clippy::missing_fields_in_debug)]
impl<R: fmt::Debug> fmt::Debug for StatIO<R> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut debug = f.debug_struct("StatIO");
		debug.field("inner", &self.inner);

		#[cfg(feature = "statistics")]
		debug.field("statistics", &self.statistics);

		debug.finish()
	}
}

impl<R> StatIO<R> {
	#[inline]
	pub fn new(reader: R) -> Self {
		Self { inner: reader, statistics: Statistics::new() }
	}
}

impl<W: Write> Write for StatIO<W> {
	type Error = W::Error;

	fn write_slice(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		let n = self.inner.write_slice(buf)?;
		mark_io_op(&mut self.statistics, n);
		Ok(n)
	}

	fn flush_once(&mut self) -> Result<(), Self::Error> {
		self.inner.flush_once()
	}
}

impl<W: AsyncWrite> AsyncWrite for StatIO<W> {
	type Error = W::Error;

	fn poll_write_slice(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, Self::Error>> {
		let this = self.project();

		let n = ready!(this.inner.poll_write_slice(cx, buf))?;
		mark_io_op(this.statistics, n);
		Poll::Ready(Ok(n))
	}

	fn poll_flush_once(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		let this = self.project();
		this.inner.poll_flush_once(cx)
	}
}

impl<R: Read> Read for StatIO<R> {
	type Error = R::Error;

	fn read_slice(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, Self::Error> {
		let n = self.inner.read_slice(buf)?;
		mark_io_op(&mut self.statistics, n);
		Ok(n)
	}
}

impl<R: AsyncRead> AsyncRead for StatIO<R> {
	type Error = R::Error;

	fn poll_read_slice(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut [u8],
	) -> Poll<Result<usize, Self::Error>> {
		let this = self.project();
		let n = ready!(this.inner.poll_read_slice(cx, buf))?;
		mark_io_op(this.statistics, n);
		Poll::Ready(Ok(n))
	}
}

fn mark_io_op(statistics: &mut Statistics, bytes: usize) {
	let bytes = bytes as u64;
	statistics.add_total_bytes(bytes);
	statistics.inc_ops();
}
