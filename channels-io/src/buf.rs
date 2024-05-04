use core::fmt;
use core::ops::{Deref, DerefMut};

#[allow(missing_debug_implementations)]
pub struct BaseBuf<T> {
	inner: T,
	pos: usize,
}

impl<T> BaseBuf<T> {
	#[inline]
	pub fn new(buf: T) -> Self {
		Self { inner: buf, pos: 0 }
	}
}

impl<T: AsRef<[u8]>> BaseBuf<T> {
	/// Get a reference to the entire buffer.
	#[inline]
	pub fn inner(&self) -> &[u8] {
		self.inner.as_ref()
	}

	#[inline]
	fn before_cursor(&self) -> &[u8] {
		&self.inner()[..self.pos]
	}

	#[inline]
	fn after_cursor(&self) -> &[u8] {
		&self.inner()[self.pos..]
	}

	/// Set the position of the inner cursor.
	///
	/// # Panics
	///
	/// If `new_pos` > `buf.len()`
	#[inline]
	#[allow(clippy::missing_safety_doc)]
	pub unsafe fn set_pos(&mut self, new_pos: usize) {
		assert!(
			new_pos <= self.inner().len(),
			"new_pos must not point outside the buffer"
		);
		self.pos = new_pos;
	}

	/// Advance the inner cursor by `n` bytes.
	///
	/// # Panics
	///
	/// If `n` will cause the cursor to go out of bounds.
	#[inline]
	pub fn advance(&mut self, n: usize) {
		if n == 0 {
			return;
		}

		unsafe { self.set_pos(usize::saturating_add(self.pos, n)) };
	}
}

impl<T: AsMut<[u8]>> BaseBuf<T> {
	/// Get a mutable reference to the entire buffer.
	#[inline]
	pub fn inner_mut(&mut self) -> &mut [u8] {
		self.inner.as_mut()
	}

	#[inline]
	fn before_cursor_mut(&mut self) -> &mut [u8] {
		let pos = self.pos;
		&mut self.inner_mut()[..pos]
	}

	#[inline]
	fn after_cursor_mut(&mut self) -> &mut [u8] {
		let pos = self.pos;
		&mut self.inner_mut()[pos..]
	}
}

/// A mutable buffer used for reading data.
///
/// This buffer has an internal cursor so it can remember how much data is
/// contained in it.
///
/// The buffer consists of 2 parts; the "filled" part and the "unfilled" part.
/// In the "filled" part lives the data that has been read into the buffer. The
/// "unfilled" part is the part of the buffer that is available for new data to
/// be placed in.
///
/// The buffer looks like this in memory:
/// ```not_rust
/// +--------------------------------+----------------------+
/// |             filled             |       unfilled       |
/// +--------------------------------+----------------------+
///                                  ^ cursor
/// ```
pub struct ReadBuf<'a> {
	inner: BaseBuf<&'a mut [u8]>,
}

impl<'a> ReadBuf<'a> {
	/// Create a new [`ReadBuf`] from `buf`.
	#[inline]
	#[must_use]
	pub fn new(buf: &'a mut [u8]) -> Self {
		Self { inner: BaseBuf::new(buf) }
	}

	/// Get a reference to the filled part of the buffer.
	#[inline]
	#[must_use]
	pub fn filled(&self) -> &[u8] {
		self.before_cursor()
	}

	/// Get a mutable reference to the filled part of the buffer.
	#[inline]
	#[must_use]
	pub fn filled_mut(&mut self) -> &mut [u8] {
		self.before_cursor_mut()
	}

	/// Get a reference to the unfilled part of the buffer.
	#[inline]
	#[must_use]
	pub fn unfilled(&self) -> &[u8] {
		self.after_cursor()
	}

	/// Get a mutable reference to the unfilled part of the buffer.
	#[inline]
	#[must_use]
	pub fn unfilled_mut(&mut self) -> &mut [u8] {
		self.after_cursor_mut()
	}
}

impl<'a> Deref for ReadBuf<'a> {
	type Target = BaseBuf<&'a mut [u8]>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<'a> DerefMut for ReadBuf<'a> {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl fmt::Debug for ReadBuf<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_list().entries(self.filled().iter()).finish()
	}
}

/// An immutable buffer used for writing data.
///
/// This buffer has an internal cursor so it can remember how much data has been
/// "taken out" of it.
///
/// The buffer consists of 2 parts; the "consumed" part and the "remaining" part.
/// The "consumed" part holds all of the data that has been "taken out" of the
/// buffer. The "remaining" parts holds all of the data that is yet to be "taken
/// out". "Taking out" data from the buffer does not change its size in memory,
/// but rather advances the internal cursor to change the size of each part.
///
/// The buffer looks like this in memory:
/// ```not_rust
/// +--------------------------------+----------------------+
/// |            consumed            |       remaining      |
/// +--------------------------------+----------------------+
///                                  ^ cursor
/// ```
pub struct WriteBuf<'a> {
	inner: BaseBuf<&'a [u8]>,
}

impl<'a> WriteBuf<'a> {
	/// Create a new [`WriteBuf`] from `buf`.
	#[inline]
	#[must_use]
	pub fn new(buf: &'a [u8]) -> Self {
		Self { inner: BaseBuf::new(buf) }
	}

	/// Get a reference to the consumed part of the buffer.
	#[inline]
	#[must_use]
	pub fn consumed(&self) -> &[u8] {
		self.before_cursor()
	}

	/// Get a reference to the remaining part of the buffer.
	#[inline]
	#[must_use]
	pub fn remaining(&self) -> &[u8] {
		self.after_cursor()
	}
}

impl<'a> Deref for WriteBuf<'a> {
	type Target = BaseBuf<&'a [u8]>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<'a> DerefMut for WriteBuf<'a> {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl fmt::Debug for WriteBuf<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_list().entries(self.remaining().iter()).finish()
	}
}
