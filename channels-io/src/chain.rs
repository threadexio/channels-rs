//! An adapter that will "chain" 2 buffers together making them act as one.

use super::{Buf, BufMut, Walkable, WalkableMut};

/// An adapter that will "chain" 2 buffers together making them act as one.
#[derive(Debug)]
pub struct Chain<A, B> {
	a: A,
	b: B,
}

/// Create a new [`Chain`] adapter by chaining `a` and `b`.
pub fn chain<A, B>(a: A, b: B) -> Chain<A, B> {
	Chain { a, b }
}

impl<A, B> Chain<A, B> {
	/// Get a reference to the first buffer in the chain.
	pub fn first(&self) -> &A {
		&self.a
	}

	/// Get a mutable reference to the first buffer in the chain.
	pub fn first_mut(&mut self) -> &mut A {
		&mut self.a
	}

	/// Get a reference to the last buffer in the chain.
	pub fn last(&self) -> &B {
		&self.b
	}

	/// Get a mutable reference to the last buffer in the chain.
	pub fn last_mut(&mut self) -> &mut B {
		&mut self.b
	}

	/// Destruct the adapter and get back the chained buffers.
	pub fn into_inner(self) -> (A, B) {
		(self.a, self.b)
	}
}

impl<A, B> Buf for Chain<A, B>
where
	A: Buf,
	B: Buf,
{
	fn chunk(&self) -> &[u8] {
		match self.a.chunk() {
			[] => self.b.chunk(),
			chunk => chunk,
		}
	}

	fn remaining(&self) -> usize {
		usize::saturating_add(self.a.remaining(), self.b.remaining())
	}

	fn advance(&mut self, mut cnt: usize) {
		if cnt == 0 {
			return;
		}

		assert!(
			cnt <= self.remaining(),
			"tried to advance past end of chain"
		);

		macro_rules! advance_buf {
			($buf:expr, $cnt:expr) => {{
				let x = core::cmp::min($cnt, $buf.remaining());
				if x != 0 {
					$buf.advance(x);
					$cnt -= x;
				}
			}};
		}

		advance_buf!(self.a, cnt);
		advance_buf!(self.b, cnt);
		let _ = cnt;
	}
}

impl<'a, A, B> Walkable<'a> for Chain<A, B>
where
	A: Walkable<'a>,
	B: Walkable<'a>,
{
	type Iter = Walk<'a, A, B>;

	fn walk_chunks(&'a self) -> Self::Iter {
		Walk::new(self)
	}
}

/// [`Chain`] walk iterator.
#[derive(Debug)]
pub struct Walk<'a, A, B>
where
	A: Walkable<'a>,
	B: Walkable<'a>,
{
	a: A::Iter,
	b: B::Iter,
}

impl<'a, A, B> Walk<'a, A, B>
where
	A: Walkable<'a>,
	B: Walkable<'a>,
{
	fn new(chain: &'a Chain<A, B>) -> Self {
		Self { a: chain.a.walk_chunks(), b: chain.b.walk_chunks() }
	}
}

impl<'a, A, B> Iterator for Walk<'a, A, B>
where
	A: Walkable<'a>,
	B: Walkable<'a>,
{
	type Item = &'a [u8];

	fn next(&mut self) -> Option<Self::Item> {
		fn get_chunk<'a, I>(buf: &mut I) -> Option<&'a [u8]>
		where
			I: Iterator<Item = &'a [u8]>,
		{
			match buf.next() {
				Some([]) | None => None,
				Some(chunk) => Some(chunk),
			}
		}

		get_chunk(&mut self.a).or_else(|| get_chunk(&mut self.b))
	}
}

impl<A, B> BufMut for Chain<A, B>
where
	A: BufMut,
	B: BufMut,
{
	fn chunk_mut(&mut self) -> &mut [u8] {
		match self.a.chunk_mut() {
			[] => self.b.chunk_mut(),
			chunk => chunk,
		}
	}

	fn remaining_mut(&self) -> usize {
		usize::saturating_add(
			self.a.remaining_mut(),
			self.b.remaining_mut(),
		)
	}

	fn advance_mut(&mut self, mut cnt: usize) {
		if cnt == 0 {
			return;
		}

		assert!(
			cnt <= self.remaining_mut(),
			"tried to advance past end of chain"
		);

		macro_rules! advance_buf {
			($buf:expr, $cnt:expr) => {{
				let x = core::cmp::min($cnt, $buf.remaining_mut());
				if x != 0 {
					$buf.advance_mut(x);
					$cnt -= x;
				}
			}};
		}

		advance_buf!(self.a, cnt);
		advance_buf!(self.b, cnt);
		let _ = cnt;
	}
}

impl<'a, A, B> WalkableMut<'a> for Chain<A, B>
where
	A: WalkableMut<'a>,
	B: WalkableMut<'a>,
{
	type Iter = WalkMut<'a, A, B>;

	fn walk_chunks_mut(&'a mut self) -> Self::Iter {
		WalkMut::new(self)
	}
}

/// [`Chain`] walk iterator.
#[derive(Debug)]
pub struct WalkMut<'a, A, B>
where
	A: WalkableMut<'a>,
	B: WalkableMut<'a>,
{
	a: A::Iter,
	b: B::Iter,
}

impl<'a, A, B> WalkMut<'a, A, B>
where
	A: WalkableMut<'a>,
	B: WalkableMut<'a>,
{
	fn new(chain: &'a mut Chain<A, B>) -> Self {
		Self {
			a: chain.a.walk_chunks_mut(),
			b: chain.b.walk_chunks_mut(),
		}
	}
}

impl<'a, A, B> Iterator for WalkMut<'a, A, B>
where
	A: WalkableMut<'a>,
	B: WalkableMut<'a>,
{
	type Item = &'a mut [u8];

	fn next(&mut self) -> Option<Self::Item> {
		fn get_chunk<'a, I>(buf: &mut I) -> Option<&'a mut [u8]>
		where
			I: Iterator<Item = &'a mut [u8]>,
		{
			match buf.next() {
				Some([]) | None => None,
				Some(chunk) => Some(chunk),
			}
		}

		get_chunk(&mut self.a).or_else(|| get_chunk(&mut self.b))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	mod buf_impl {
		use super::{chain, Buf, Walkable};

		type Chain<'a> = super::Chain<&'a [u8], &'a [u8]>;

		#[test]
		fn basic() {
			let mut chain: Chain =
				chain(&[0, 1, 2, 3], &[4, 5, 6, 7, 8, 9]);

			assert_eq!(chain.remaining(), 10);
			assert_eq!(chain.chunk(), &[0, 1, 2, 3]);
			assert!(chain.walk_chunks().eq([
				[0, 1, 2, 3].as_slice(),
				[4, 5, 6, 7, 8, 9].as_slice()
			]));

			chain.advance(3);
			assert_eq!(chain.remaining(), 7);
			assert_eq!(chain.chunk(), [3]);
			assert!(chain
				.walk_chunks()
				.eq([[3].as_slice(), [4, 5, 6, 7, 8, 9].as_slice()]));

			chain.advance(4);
			assert_eq!(chain.remaining(), 3);
			assert_eq!(chain.chunk(), [7, 8, 9]);
			assert!(chain.walk_chunks().eq([[7, 8, 9].as_slice()]));
		}

		#[test]
		fn empty_a() {
			let mut chain: Chain = chain(&[], &[4, 5, 6, 7, 8, 9]);

			assert_eq!(chain.remaining(), 6);
			assert_eq!(chain.chunk(), [4, 5, 6, 7, 8, 9]);
			assert!(chain
				.walk_chunks()
				.eq([[4, 5, 6, 7, 8, 9].as_slice()]));

			chain.advance(3);
			assert_eq!(chain.remaining(), 3);
			assert_eq!(chain.chunk(), [7, 8, 9]);
			assert!(chain.walk_chunks().eq([[7, 8, 9].as_slice()]));
		}

		#[test]
		fn empty_b() {
			let mut chain: Chain = chain(&[0, 1, 2, 3], &[]);

			assert_eq!(chain.remaining(), 4);
			assert_eq!(chain.chunk(), [0, 1, 2, 3]);
			assert!(chain
				.walk_chunks()
				.eq([[0, 1, 2, 3].as_slice()]));

			chain.advance(3);
			assert_eq!(chain.remaining(), 1);
			assert_eq!(chain.chunk(), [3]);
			assert!(chain.walk_chunks().eq([[3].as_slice()]));
		}

		#[test]
		fn empty() {
			let chain: Chain = chain(&[], &[]);

			assert_eq!(chain.remaining(), 0);
			assert_eq!(chain.chunk(), &[]);
			assert!(chain.walk_chunks().eq::<[&[u8]; 0]>([]));
		}

		#[test]
		fn advance_all() {
			let mut chain: Chain =
				chain(&[0, 1, 2, 3], &[4, 5, 6, 7, 8, 9]);

			chain.advance(10);
			assert_eq!(chain.remaining(), 0);
			assert_eq!(chain.chunk(), &[]);
		}

		#[test]
		fn advance_to_between() {
			let mut chain: Chain =
				chain(&[0, 1, 2, 3], &[4, 5, 6, 7, 8, 9]);

			assert_eq!(chain.remaining(), 10);
			assert_eq!(chain.chunk(), [0, 1, 2, 3]);

			chain.advance(4);
			assert_eq!(chain.remaining(), 6);
			assert_eq!(chain.chunk(), [4, 5, 6, 7, 8, 9]);
			assert!(chain
				.walk_chunks()
				.eq([[4, 5, 6, 7, 8, 9].as_slice()]));

			chain.advance(2);
			assert_eq!(chain.remaining(), 4);
			assert_eq!(chain.chunk(), [6, 7, 8, 9]);
			assert!(chain
				.walk_chunks()
				.eq([[6, 7, 8, 9].as_slice()]));
		}

		#[test]
		#[should_panic(
			expected = "tried to advance past end of chain"
		)]
		fn panic_on_advance_out_of_bounds() {
			let mut chain: Chain =
				chain(&[0, 1, 2, 3], &[4, 5, 6, 7, 8, 9]);

			chain.advance(11);
		}
	}

	mod bufmut_impl {
		use super::{chain, BufMut, WalkableMut};

		type Chain<'a> = super::Chain<&'a mut [u8], &'a mut [u8]>;

		#[test]
		fn basic() {
			let mut a = [0, 1, 2, 3];
			let mut b = [4, 5, 6, 7, 8, 9];
			let mut chain: Chain = chain(&mut a, &mut b);

			assert_eq!(chain.remaining_mut(), 10);
			assert_eq!(chain.chunk_mut(), [0, 1, 2, 3]);
			assert!(chain.walk_chunks_mut().eq([
				[0, 1, 2, 3].as_slice(),
				[4, 5, 6, 7, 8, 9].as_slice()
			]));

			chain.advance_mut(3);
			assert_eq!(chain.remaining_mut(), 7);
			assert_eq!(chain.chunk_mut(), [3]);
			assert!(chain
				.walk_chunks_mut()
				.eq([[3].as_slice(), [4, 5, 6, 7, 8, 9].as_slice()]));

			chain.advance_mut(4);
			assert_eq!(chain.remaining_mut(), 3);
			assert_eq!(chain.chunk_mut(), [7, 8, 9]);
		}

		#[test]
		fn empty_a() {
			let mut a = [];
			let mut b = [4, 5, 6, 7, 8, 9];
			let mut chain: Chain = chain(&mut a, &mut b);

			assert_eq!(chain.remaining_mut(), 6);
			assert_eq!(chain.chunk_mut(), [4, 5, 6, 7, 8, 9]);
			assert!(chain
				.walk_chunks_mut()
				.eq([[4, 5, 6, 7, 8, 9].as_slice()]));

			chain.advance_mut(3);
			assert_eq!(chain.remaining_mut(), 3);
			assert_eq!(chain.chunk_mut(), [7, 8, 9]);
			assert!(chain
				.walk_chunks_mut()
				.eq([[7, 8, 9].as_slice()]));
		}

		#[test]
		fn empty_b() {
			let mut a = [0, 1, 2, 3];
			let mut b = [];
			let mut chain: Chain = chain(&mut a, &mut b);

			assert_eq!(chain.remaining_mut(), 4);
			assert_eq!(chain.chunk_mut(), [0, 1, 2, 3]);
			assert!(chain
				.walk_chunks_mut()
				.eq([[0, 1, 2, 3].as_slice()]));

			chain.advance_mut(3);
			assert_eq!(chain.remaining_mut(), 1);
			assert_eq!(chain.chunk_mut(), [3]);
			assert!(chain.walk_chunks_mut().eq([[3].as_slice()]));
		}

		#[test]
		fn empty() {
			let mut a = [];
			let mut b = [];
			let mut chain: Chain = chain(&mut a, &mut b);

			assert_eq!(chain.remaining_mut(), 0);
			assert_eq!(chain.chunk_mut(), &[]);
			assert!(chain.walk_chunks_mut().eq::<[&[u8]; 0]>([]));
		}

		#[test]
		fn advance_all() {
			let mut a = [0, 1, 2, 3];
			let mut b = [4, 5, 6, 7, 8, 9];
			let mut chain: Chain = chain(&mut a, &mut b);

			chain.advance_mut(10);
			assert_eq!(chain.remaining_mut(), 0);
			assert_eq!(chain.chunk_mut(), &[]);
			assert!(chain.walk_chunks_mut().eq::<[&[u8]; 0]>([]));
		}

		#[test]
		fn advance_to_between() {
			let mut a = [0, 1, 2, 3];
			let mut b = [4, 5, 6, 7, 8, 9];
			let mut chain: Chain = chain(&mut a, &mut b);

			assert_eq!(chain.remaining_mut(), 10);
			assert_eq!(chain.chunk_mut(), [0, 1, 2, 3]);
			assert!(chain.walk_chunks_mut().eq([
				[0, 1, 2, 3].as_slice(),
				[4, 5, 6, 7, 8, 9].as_slice()
			]));

			chain.advance_mut(4);
			assert_eq!(chain.remaining_mut(), 6);
			assert_eq!(chain.chunk_mut(), [4, 5, 6, 7, 8, 9]);
			assert!(chain
				.walk_chunks_mut()
				.eq([[4, 5, 6, 7, 8, 9].as_slice()]));

			chain.advance_mut(2);
			assert_eq!(chain.remaining_mut(), 4);
			assert_eq!(chain.chunk_mut(), [6, 7, 8, 9]);
			assert!(chain
				.walk_chunks_mut()
				.eq([[6, 7, 8, 9].as_slice()]));
		}

		#[test]
		#[should_panic(
			expected = "tried to advance past end of chain"
		)]
		fn panic_on_advance_out_of_bounds() {
			let mut a = [0, 1, 2, 3];
			let mut b = [4, 5, 6, 7, 8, 9];
			let mut chain: Chain = chain(&mut a, &mut b);

			chain.advance_mut(11);
		}
	}
}
