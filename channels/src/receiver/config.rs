use core::fmt;
use core::num::NonZeroUsize;

/// Configuration for [`Receiver`].
#[derive(Clone)]
#[must_use = "`Config`s don't do anything on their own"]
pub struct Config {
	pub(crate) max_size: Option<NonZeroUsize>,
	pub(crate) flags: u8,
}

impl Config {
	const VERIFY_ORDER: u8 = 1 << 0;

	#[inline]
	const fn get_flag(&self, flag: u8) -> bool {
		self.flags & flag != 0
	}

	#[inline]
	fn set_flag(&mut self, flag: u8, value: bool) {
		if value {
			self.flags |= flag;
		} else {
			self.flags &= !flag;
		}
	}
}

impl Default for Config {
	#[inline]
	fn default() -> Self {
		Self { flags: Self::VERIFY_ORDER, max_size: None }
	}
}

impl Config {
	/// Get the max payload size the [`Receiver`] will accept.
	#[inline]
	#[must_use]
	pub fn max_size(&self) -> usize {
		self.max_size.map_or(0, NonZeroUsize::get)
	}

	/// Set the max payload size the [`Receiver`] will accept.
	#[allow(clippy::missing_panics_doc)]
	#[inline]
	pub fn set_max_size(&mut self, max_size: usize) -> &mut Self {
		self.max_size = match max_size {
			0 => None,
			x => Some(
				NonZeroUsize::new(x)
					.expect("max_size should never be 0"),
			),
		};
		self
	}

	/// Set the max payload size the [`Receiver`] will accept.
	#[inline]
	pub fn with_max_size(mut self, max_size: usize) -> Self {
		self.set_max_size(max_size);
		self
	}

	/// Check whether the [`Receiver`] will verify the order of received frames.
	#[inline]
	#[must_use]
	pub fn verify_order(&self) -> bool {
		self.get_flag(Self::VERIFY_ORDER)
	}

	/// Set whether the [`Receiver`] will verify the order of received frames.
	pub fn set_verify_order(&mut self, yes: bool) -> &mut Self {
		self.set_flag(Self::VERIFY_ORDER, yes);
		self
	}

	/// Set whether the [`Receiver`] will verify the order of received frames.
	pub fn with_verify_order(mut self, yes: bool) -> Self {
		self.set_verify_order(yes);
		self
	}
}

impl fmt::Debug for Config {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Config")
			.field("max_size", &self.max_size())
			.field("verify_order", &self.verify_order())
			.finish()
	}
}
