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
	pub fn limit(&self) -> usize {
		self.left
	}

	/// Set the maximum number of bytes this adapter allows to be written.
	pub fn set_limit(&mut self, limit: usize) {
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

impl<'a, B> WalkableMut<'a> for Limit<B>
where
	B: WalkableMut<'a>,
{
	type Iter = Walk<'a, B>;

	fn walk_chunks_mut(&'a mut self) -> Self::Iter {
		Walk::new(self)
	}
}

#[derive(Debug)]
pub struct Walk<'a, B>
where
	B: WalkableMut<'a>,
{
	chunks: B::Iter,
	left: usize,
}

impl<'a, B> Walk<'a, B>
where
	B: WalkableMut<'a>,
{
	fn new(limit: &'a mut Limit<B>) -> Self {
		Self { chunks: limit.buf.walk_chunks_mut(), left: limit.left }
	}
}

impl<'a, B> Iterator for Walk<'a, B>
where
	B: WalkableMut<'a>,
{
	type Item = &'a mut [u8];

	fn next(&mut self) -> Option<Self::Item> {
		if self.left == 0 {
			return None;
		}

		match self.chunks.next()? {
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
