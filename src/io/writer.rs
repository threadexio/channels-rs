use core::any::type_name;
use core::fmt;

use std::io;

pub struct Writer<W> {
	pub(super) inner: W,

	#[cfg(feature = "statistics")]
	pub stats: crate::stats::SendStats,
}

impl<W> Writer<W> {
	pub fn new(writer: W) -> Self {
		Self {
			inner: writer,

			#[cfg(feature = "statistics")]
			stats: crate::stats::SendStats::new(),
		}
	}

	pub fn get(&self) -> &W {
		&self.inner
	}

	pub fn get_mut(&mut self) -> &mut W {
		&mut self.inner
	}

	pub(super) fn on_write(
		&mut self,
		_buf: &[u8],
		n: usize,
	) -> io::Result<()> {
		#[cfg(feature = "statistics")]
		self.stats.add_sent(n);

		Ok(())
	}
}

impl<W> fmt::Debug for Writer<W> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_struct("Writer");
		s.field("inner", &type_name::<W>());

		#[cfg(feature = "statistics")]
		s.field("stats", &self.stats);

		s.finish()
	}
}
