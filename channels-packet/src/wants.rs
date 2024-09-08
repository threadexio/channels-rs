/// Signal to the calling code that parsing cannot continue until this many more bytes have
/// been read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Wants(pub(crate) usize);

impl Wants {
	/// Get the number of bytes required before parsing can make progress.
	#[inline]
	#[must_use]
	pub fn get(&self) -> usize {
		self.0
	}
}
