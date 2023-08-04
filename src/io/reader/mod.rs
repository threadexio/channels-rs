use core::any::type_name;
use core::fmt;

use crate::macros::*;

pub struct Reader<R> {
	inner: R,

	#[cfg(feature = "statistics")]
	pub stats: crate::stats::RecvStats,
}

impl<R> Reader<R> {
	pub fn new(reader: R) -> Self {
		Self {
			inner: reader,

			#[cfg(feature = "statistics")]
			stats: crate::stats::RecvStats::new(),
		}
	}

	pub fn get(&self) -> &R {
		&self.inner
	}

	pub fn get_mut(&mut self) -> &mut R {
		&mut self.inner
	}
}

impl<R> fmt::Debug for Reader<R> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_struct("Reader");
		s.field("inner", &type_name::<R>());

		#[cfg(feature = "statistics")]
		s.field("stats", &self.stats);

		s.finish()
	}
}

mod blocking;

cfg_tokio! {
	mod tokio;
}
