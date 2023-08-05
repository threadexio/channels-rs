use core::any::type_name;
use core::fmt;

pub struct Writer<W> {
	inner: W,

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

mod blocking;

cfg_tokio! {
	mod tokio;
}
