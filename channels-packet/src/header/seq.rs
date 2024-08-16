use crate::num::u6;

/// An endless sequence of consecutive frame numbers.
///
/// This sequence will yield frame numbers for consecutive frames. Each frame number is
/// calculated by incrementing the number of the previous frame by one and wrapping around
/// on overflow. Being 6 bits in size, frame numbers can only go up to 63 and thus the sequence
/// has a period of 64. Skipping frame numbers is considered bad practice.
#[derive(Debug)]
pub struct FrameNumSequence {
	state: u8,
}

impl FrameNumSequence {
	/// Create a new sequence that starts at `x`.
	#[inline]
	#[must_use]
	pub const fn starting_at(x: u6) -> Self {
		Self { state: x.get() }
	}

	/// Create a new frame number sequence starting at 0.
	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		Self::starting_at(u6::new_truncate(0))
	}

	/// Return the next frame number in the sequence and advance it by one.
	#[inline]
	pub fn advance(&mut self) -> u6 {
		let x = self.peek();
		self.state = self.state.wrapping_add(1);
		x
	}

	/// Peek at the next frame number in the sequence without advancing it.
	#[inline]
	#[must_use]
	pub const fn peek(&self) -> u6 {
		self.peek_n(1)
	}

	/// Peek at the n-th next frame number in the sequence without advancing it.
	#[allow(clippy::cast_possible_truncation)]
	#[inline]
	#[must_use]
	pub const fn peek_n(&self, n: usize) -> u6 {
		assert!(n >= 1, "n must be greater or equal to 1");
		// TODO: safety
		u6::new_truncate(self.state.wrapping_add((n - 1) as u8))
	}
}

impl Default for FrameNumSequence {
	fn default() -> Self {
		Self::new()
	}
}

impl Iterator for FrameNumSequence {
	type Item = u6;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		Some(self.advance())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const N: usize = 512;

	#[test]
	#[allow(clippy::cast_possible_truncation)]
	fn test_sequence() {
		let seq = FrameNumSequence::new().take(N);
		let expected = (0..N).map(|x| x as u8).map(u6::new_truncate);

		assert!(seq.eq(expected));
	}

	#[test]
	fn test_sequence_peek() {
		let mut seq = FrameNumSequence::new();

		for _ in 0..N {
			let peeked = seq.peek();
			let next = seq.advance();
			assert_eq!(peeked, next);
		}
	}
}
