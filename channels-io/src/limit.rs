//! An adapter for [`BufMut`] that will allow writing only up to a specific
//! number of bytes.

use super::{BufMut, ContiguousMut, WalkableMut};

/// An adapter for [`BufMut`] that will allow writing only up to a specific
/// number of bytes.
#[derive(Debug)]
pub struct Limit<B> {
	buf: B,
	left: usize,
}

/// Create a new [`Limit`] adapter that will be able to write at most `limit`
/// bytes.
pub fn limit<B>(buf: B, limit: usize) -> Limit<B> {
	Limit { buf, left: limit }
}

impl<B> Limit<B> {
	/// Get a reference to the internal buffer.
	pub fn get(&self) -> &B {
		&self.buf
	}

	/// Get a mutable reference to the internal buffer.
	pub fn get_mut(&mut self) -> &mut B {
		&mut self.buf
	}

	/// Destruct the adapter and get back the internal buffer.
	pub fn into_inner(self) -> B {
		self.buf
	}

	/// Get the maximum number of bytes this adapter allows to be written.
	pub fn left(&self) -> usize {
		self.left
	}

	/// Set the maximum number of bytes this adapter allows to be written.
	pub fn set_left(&mut self, limit: usize) {
		self.left = limit;
	}
}

impl<B> BufMut for Limit<B>
where
	B: BufMut,
{
	fn chunk_mut(&mut self) -> &mut [u8] {
		match self.buf.chunk_mut() {
			chunk if chunk.len() > self.left => {
				&mut chunk[..self.left]
			},
			chunk => chunk,
		}
	}

	fn remaining_mut(&self) -> usize {
		core::cmp::min(self.left, self.buf.remaining_mut())
	}

	fn advance_mut(&mut self, cnt: usize) {
		assert!(
			cnt <= self.remaining_mut(),
			"tried to advance past end of limit"
		);
		self.buf.advance_mut(cnt);
		self.left -= cnt;
	}
}

unsafe impl<B> ContiguousMut for Limit<B> where B: ContiguousMut {}

impl<B> WalkableMut for Limit<B>
where
	B: WalkableMut,
{
	type Iter<'a> = Walk<'a, B>
	where
		Self: 'a;

	fn walk_chunks_mut(&mut self) -> Self::Iter<'_> {
		Walk::new(self)
	}
}

/// [`Limit`] walk iterator.
#[derive(Debug)]
pub struct Walk<'a, B>
where
	B: WalkableMut + 'a,
{
	chunks: B::Iter<'a>,
	left: usize,
}

impl<'a, B> Walk<'a, B>
where
	B: WalkableMut,
{
	fn new(limit: &'a mut Limit<B>) -> Self {
		Self { chunks: limit.buf.walk_chunks_mut(), left: limit.left }
	}
}

impl<'a, B> Iterator for Walk<'a, B>
where
	B: WalkableMut,
{
	type Item = &'a mut [u8];

	fn next(&mut self) -> Option<Self::Item> {
		match self.chunks.next()? {
			[] => None,
			_ if self.left == 0 => None,
			chunk if chunk.len() > self.left => {
				let ret = &mut chunk[..self.left];
				self.left = 0;
				Some(ret)
			},
			chunk => {
				self.left -= chunk.len();
				Some(chunk)
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{limit, BufMut, WalkableMut};

	type Limit<'a> = super::Limit<&'a mut [u8]>;

	#[test]
	fn more_than_left() {
		let mut a = [0, 1, 2, 3, 4, 5, 6, 7];
		let mut limit: Limit = limit(&mut a, 5);

		assert_eq!(limit.left(), 5);
		assert_eq!(limit.remaining_mut(), 5);
		assert_eq!(limit.chunk_mut(), [0, 1, 2, 3, 4]);
		assert!(limit
			.walk_chunks_mut()
			.eq([[0, 1, 2, 3, 4].as_slice()]));

		limit.advance_mut(3);
		assert_eq!(limit.left(), 2);
		assert_eq!(limit.remaining_mut(), 2);
		assert_eq!(limit.chunk_mut(), [3, 4]);
		assert!(limit.walk_chunks_mut().eq([[3, 4].as_slice()]));

		limit.advance_mut(2);
		assert_eq!(limit.left(), 0);
		assert_eq!(limit.remaining_mut(), 0);
		assert_eq!(limit.chunk_mut(), []);
		assert_eq!(limit.walk_chunks_mut().next(), None);

		limit.set_left(2);
		assert_eq!(limit.left(), 2);
		assert_eq!(limit.remaining_mut(), 2);
		assert_eq!(limit.chunk_mut(), [5, 6]);
		assert!(limit.walk_chunks_mut().eq([[5, 6].as_slice()]));
	}

	#[test]
	fn less_than_left() {
		let mut a = [0, 1, 2];
		let mut limit: Limit = limit(&mut a, 5);

		assert_eq!(limit.left(), 5);
		assert_eq!(limit.remaining_mut(), 3);
		assert_eq!(limit.chunk_mut(), [0, 1, 2]);
		assert!(limit.walk_chunks_mut().eq([[0, 1, 2].as_slice()]));

		limit.advance_mut(3);
		assert_eq!(limit.left(), 2);
		assert_eq!(limit.remaining_mut(), 0);
		assert_eq!(limit.chunk_mut(), []);
		assert_eq!(limit.walk_chunks_mut().next(), None);
	}

	#[test]
	fn equal_to_left() {
		let mut a = [0, 1, 2];
		let mut limit: Limit = limit(&mut a, 3);

		assert_eq!(limit.left(), 3);
		assert_eq!(limit.remaining_mut(), 3);
		assert_eq!(limit.chunk_mut(), [0, 1, 2]);
		assert!(limit.walk_chunks_mut().eq([[0, 1, 2].as_slice()]));

		limit.advance_mut(3);
		assert_eq!(limit.left(), 0);
		assert_eq!(limit.remaining_mut(), 0);
		assert_eq!(limit.chunk_mut(), []);
		assert_eq!(limit.walk_chunks_mut().next(), None);
	}

	#[test]
	#[should_panic(expected = "tried to advance past end of limit")]
	fn advance_out_of_bounds() {
		let mut a = [0, 1, 2, 3, 4, 5, 6, 7];
		let mut limit: Limit = limit(&mut a, 5);

		limit.advance_mut(7);
	}
}
