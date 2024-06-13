use core::fmt;

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
///
/// ```not_rust
/// ┌────────────────────────────────┬──────────────────────┐
/// │             cursor             │       unfilled       │
/// └────────────────────────────────▲──────────────────────┘
///                                  └ cursor
/// ```
pub struct ReadBuf<'a> {
	inner: &'a mut [u8],
	pos: usize,
}

impl<'a> ReadBuf<'a> {
	/// Create a new [`ReadBuf`] from `buf`.
	#[inline]
	#[must_use]
	pub fn new(buf: &'a mut [u8]) -> Self {
		Self { inner: buf, pos: 0 }
	}

	/// Get a reference to the filled part of the buffer.
	#[inline]
	#[must_use]
	pub fn filled(&self) -> &[u8] {
		&self.inner[..self.pos]
	}

	/// Get a mutable reference to the filled part of the buffer.
	#[inline]
	#[must_use]
	pub fn filled_mut(&mut self) -> &mut [u8] {
		&mut self.inner[..self.pos]
	}

	/// Get a reference to the unfilled part of the buffer.
	#[inline]
	#[must_use]
	pub fn unfilled(&self) -> &[u8] {
		&self.inner[self.pos..]
	}

	/// Get a mutable reference to the unfilled part of the buffer.
	#[inline]
	#[must_use]
	pub fn unfilled_mut(&mut self) -> &mut [u8] {
		&mut self.inner[self.pos..]
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
			new_pos <= self.inner.len(),
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

	/// Advance the inner cursor by `n` bytes and return the bytes between the
	/// old and the new cursor as a slice.
	///
	/// # Panics
	///
	/// If `n` will cause the cursor to go out of bounds.
	#[inline]
	pub fn consume(&mut self, n: usize) -> &mut [u8] {
		if n == 0 {
			return &mut [];
		}

		let old_pos = self.pos;
		self.advance(n);
		let new_pos = self.pos;

		&mut self.inner[old_pos..new_pos]
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
///
/// ```not_rust
/// ┌────────────────────────────────┬──────────────────────┐
/// │            consumed            │       remaining      │
/// └────────────────────────────────▲──────────────────────┘
///                                  └ cursor
/// ```
pub struct WriteBuf<'a> {
	inner: &'a [u8],
	pos: usize,
}

impl<'a> WriteBuf<'a> {
	/// Create a new [`WriteBuf`] from `buf`.
	#[inline]
	#[must_use]
	pub fn new(buf: &'a [u8]) -> Self {
		Self { inner: buf, pos: 0 }
	}

	/// Get a reference to the consumed part of the buffer.
	#[inline]
	#[must_use]
	pub fn consumed(&self) -> &'a [u8] {
		&self.inner[..self.pos]
	}

	/// Get a reference to the remaining part of the buffer.
	#[inline]
	#[must_use]
	pub fn remaining(&self) -> &'a [u8] {
		&self.inner[self.pos..]
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
			new_pos <= self.inner.len(),
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

	/// Advance the inner cursor by `n` bytes and return the bytes between the
	/// old and the new cursor as a slice.
	///
	/// # Panics
	///
	/// If `n` will cause the cursor to go out of bounds.
	#[inline]
	pub fn consume(&mut self, n: usize) -> &'a [u8] {
		if n == 0 {
			return &[];
		}

		let old_pos = self.pos;
		self.advance(n);
		let new_pos = self.pos;

		&self.inner[old_pos..new_pos]
	}
}

impl fmt::Debug for WriteBuf<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_list().entries(self.remaining().iter()).finish()
	}
}
