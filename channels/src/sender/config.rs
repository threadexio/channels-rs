use core::fmt;

/// Configuration for [`Sender`].
///
/// [`Sender`]: super::Sender
#[derive(Clone)]
#[must_use = "`Config`s don't do anything on their own"]
pub struct Config {}

impl Default for Config {
	#[inline]
	fn default() -> Self {
		Self {}
	}
}

impl fmt::Debug for Config {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Config").finish()
	}
}
