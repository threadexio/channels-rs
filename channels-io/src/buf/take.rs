use super::{Buf, Contiguous, Walkable};

#[derive(Debug)]
pub struct Take<B> {
	buf: B,
	left: usize,
}

pub fn take<B>(buf: B, limit: usize) -> Take<B>
where
	B: Buf,
{
	Take { buf, left: limit }
}

impl<B> Take<B> {
	pub fn get(&self) -> &B {
		&self.buf
	}

	pub fn get_mut(&mut self) -> &mut B {
		&mut self.buf
	}

	pub fn into_inner(self) -> B {
		self.buf
	}

	pub fn limit(&self) -> usize {
		self.left
	}

	pub fn set_limit(&mut self, limit: usize) {
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
		if self.left == 0 {
			return None;
		}

		match self.chunks.next()? {
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
