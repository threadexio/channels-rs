use core::fmt;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use crate::io::{AsyncRead, AsyncWrite, Read, Write};

#[cfg(feature = "statistics")]
mod real {
	use core::fmt;

	#[derive(Clone, Default, PartialEq, Eq, Hash)]
	pub(super) struct StatisticsImpl {
		total_bytes: u64,
		packets: u64,
		ops: u64,
	}

	impl StatisticsImpl {
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

		#[inline]
		pub fn total_bytes(&self) -> u64 {
			self.total_bytes
		}

		#[inline]
		pub fn packets(&self) -> u64 {
			self.packets
		}

		#[inline]
		pub fn ops(&self) -> u64 {
			self.ops
		}
	}

	impl fmt::Debug for StatisticsImpl {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Statistics")
				.field("total_bytes", &self.total_bytes)
				.field("packets", &self.packets)
				.field("ops", &self.ops)
				.finish()
		}
	}
}

#[cfg(not(feature = "statistics"))]
mod mock {
	#[derive(Clone, Default, PartialEq, Eq, Hash)]
	pub(super) struct StatisticsImpl;

	impl StatisticsImpl {
		pub(crate) fn add_total_bytes(&mut self, _: u64) {}
		pub(crate) fn inc_packets(&mut self) {}
		pub(crate) fn inc_ops(&mut self) {}

		pub(crate) fn total_bytes(&self) -> u64 {
			0
		}

		pub(crate) fn packets(&self) -> u64 {
			0
		}

		pub(crate) fn ops(&self) -> u64 {
			0
		}
	}
}

#[cfg(feature = "statistics")]
use self::real::StatisticsImpl;

#[cfg(not(feature = "statistics"))]
use self::mock::StatisticsImpl;

/// IO statistic information.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Statistics {
	inner: StatisticsImpl,
}

impl Statistics {
	pub(crate) fn new() -> Self {
		Self { inner: StatisticsImpl::default() }
	}

	#[inline]
	pub(crate) fn add_total_bytes(&mut self, n: u64) {
		self.inner.add_total_bytes(n);
	}

	#[inline]
	pub(crate) fn inc_packets(&mut self) {
		self.inner.inc_packets();
	}

	#[inline]
	pub(crate) fn inc_ops(&mut self) {
		self.inner.inc_ops();
	}

	/// Returns the number of bytes transferred through this reader/writer.
	#[inline]
	#[must_use]
	pub fn total_bytes(&self) -> u64 {
		self.inner.total_bytes()
	}

	/// Returns the number of packets transferred through this reader/writer.
	#[inline]
	#[must_use]
	pub fn packets(&self) -> u64 {
		self.inner.packets()
	}

	/// Returns the total number of `send`/`recv` operations.
	#[inline]
	#[must_use]
	pub fn ops(&self) -> u64 {
		self.inner.ops()
	}
}

impl fmt::Debug for Statistics {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Statistics")
			.field("total_bytes", &self.total_bytes())
			.field("packets", &self.packets())
			.field("ops", &self.ops())
			.finish()
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StatIO<R> {
	pub(crate) inner: R,
	pub(crate) statistics: Statistics,
}

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

	#[inline]
	fn on_read(&mut self, n: u64) {
		self.statistics.add_total_bytes(n);
	}

	#[inline]
	fn on_write(&mut self, n: u64) {
		self.statistics.add_total_bytes(n);
	}
}

impl<W: Write> Write for StatIO<W> {
	type Error = W::Error;

	fn write_slice(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		let n = self.inner.write_slice(buf)?;
		self.on_write(n as u64);
		Ok(n)
	}

	fn flush_once(&mut self) -> Result<(), Self::Error> {
		self.inner.flush_once()
	}
}

impl<W: AsyncWrite> AsyncWrite for StatIO<W> {
	type Error = W::Error;

	fn poll_write_slice(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, Self::Error>> {
		let n = ready!(
			Pin::new(&mut self.inner).poll_write_slice(cx, buf)
		)?;
		self.on_write(n as u64);
		Poll::Ready(Ok(n))
	}

	fn poll_flush_once(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		Pin::new(&mut self.inner).poll_flush_once(cx)
	}
}

impl<R: Read> Read for StatIO<R> {
	type Error = R::Error;

	fn read_slice(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, Self::Error> {
		let n = self.inner.read_slice(buf)?;
		self.on_read(n as u64);
		Ok(n)
	}
}

impl<R: AsyncRead> AsyncRead for StatIO<R> {
	type Error = R::Error;

	fn poll_read_slice(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut [u8],
	) -> Poll<Result<usize, Self::Error>> {
		let n = ready!(
			Pin::new(&mut self.inner).poll_read_slice(cx, buf)
		)?;
		self.on_read(n as u64);
		Poll::Ready(Ok(n))
	}
}
