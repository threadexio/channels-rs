//! An adapter for [`Buf`] that will allow reading up to a specific number of
//! bytes.

use super::{Buf, Contiguous, Walkable};

/// An adapter for [`Buf`] that will allow reading up to a specific number of
/// bytes.
#[derive(Debug)]
pub struct Take<B> {
	buf: B,
	left: usize,
}

/// Create a [`Take`] adapter that will "see" up to `limit` bytes of `buf`.
pub fn take<B>(buf: B, limit: usize) -> Take<B>
where
	B: Buf,
{
	Take { buf, left: limit }
}

impl<B> Take<B> {
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

	/// Get the maximum number of bytes this adapter will "see".
	pub fn left(&self) -> usize {
		self.left
	}

	/// Set the maximum number of bytes this adapter will "see".
	pub fn set_left(&mut self, limit: usize) {
		self.left = limit;
	}
}

impl<B> Buf for Take<B>
where
	B: Buf,
{
	fn chunk(&self) -> &[u8] {
		let rem = self.remaining();
		match self.buf.chunk() {
			chunk if chunk.len() > rem => &chunk[..rem],
			chunk => chunk,
		}
	}

	fn remaining(&self) -> usize {
		core::cmp::min(self.left, self.buf.remaining())
	}

	fn advance(&mut self, cnt: usize) {
		assert!(
			cnt <= self.remaining(),
			"tried to advance past end of take"
		);
		self.buf.advance(cnt);
		self.left -= cnt;
	}
}

unsafe impl<B> Contiguous for Take<B> where B: Contiguous {}

impl<'a, B> Walkable<'a> for Take<B>
where
	B: Walkable<'a>,
{
	type Iter = Walk<'a, B>;

	fn walk_chunks(&'a self) -> Self::Iter {
		Walk::new(self)
	}
}

/// [`Take`] walk iterator.
#[derive(Debug)]
pub struct Walk<'a, B>
where
	B: Walkable<'a>,
{
	chunks: B::Iter,
	left: usize,
}

impl<'a, B> Walk<'a, B>
where
	B: Walkable<'a>,
{
	fn new(take: &'a Take<B>) -> Self {
		Self { chunks: take.buf.walk_chunks(), left: take.left }
	}
}

impl<'a, B> Iterator for Walk<'a, B>
where
	B: Walkable<'a>,
{
	type Item = &'a [u8];

	fn next(&mut self) -> Option<Self::Item> {
		match self.chunks.next()? {
			[] => None,
			_ if self.left == 0 => None,
			chunk if chunk.len() > self.left => {
				let ret = &chunk[..self.left];
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
	use super::{take, Buf, Walkable};

	type Take<'a> = super::Take<&'a [u8]>;

	#[test]
	fn more_than_left() {
		let mut take: Take = take(&[0, 1, 2, 3, 4, 5, 6, 7], 5);

		assert_eq!(take.left(), 5);
		assert_eq!(take.remaining(), 5);
		assert_eq!(take.chunk(), [0, 1, 2, 3, 4]);
		assert!(take.walk_chunks().eq([[0, 1, 2, 3, 4].as_slice()]));

		take.advance(3);
		assert_eq!(take.left(), 2);
		assert_eq!(take.remaining(), 2);
		assert_eq!(take.chunk(), [3, 4]);
		assert!(take.walk_chunks().eq([[3, 4].as_slice()]));

		take.advance(2);
		assert_eq!(take.left(), 0);
		assert_eq!(take.remaining(), 0);
		assert_eq!(take.chunk(), []);
		assert_eq!(take.walk_chunks().next(), None);

		take.set_left(2);
		assert_eq!(take.left(), 2);
		assert_eq!(take.remaining(), 2);
		assert_eq!(take.chunk(), [5, 6]);
		assert!(take.walk_chunks().eq([[5, 6].as_slice()]));
	}

	#[test]
	fn less_than_left() {
		let mut take: Take = take(&[0, 1, 2], 5);

		assert_eq!(take.left(), 5);
		assert_eq!(take.remaining(), 3);
		assert_eq!(take.chunk(), [0, 1, 2]);
		assert!(take.walk_chunks().eq([[0, 1, 2].as_slice()]));

		take.advance(3);
		assert_eq!(take.left(), 2);
		assert_eq!(take.remaining(), 0);
		assert_eq!(take.chunk(), []);
		assert_eq!(take.walk_chunks().next(), None);
	}

	#[test]
	fn equal_to_left() {
		let mut take: Take = take(&[0, 1, 2], 3);

		assert_eq!(take.left(), 3);
		assert_eq!(take.remaining(), 3);
		assert_eq!(take.chunk(), [0, 1, 2]);
		assert!(take.walk_chunks().eq([[0, 1, 2].as_slice()]));

		take.advance(3);
		assert_eq!(take.left(), 0);
		assert_eq!(take.remaining(), 0);
		assert_eq!(take.chunk(), []);
		assert_eq!(take.walk_chunks().next(), None);
	}

	#[test]
	#[should_panic(expected = "tried to advance past end of take")]
	fn advance_out_of_bounds() {
		let mut take: Take = take(&[0, 1, 2, 3, 4, 5, 6, 7], 5);

		take.advance(7);
	}
}
