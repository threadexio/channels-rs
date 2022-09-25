use crate::prelude::*;

pub struct Buffer {
	inner: Vec<u8>,
	cursor: usize,
}

#[allow(dead_code)]
impl Buffer {
	pub fn with_size(s: usize) -> Self {
		Self { inner: vec![0u8; s], cursor: 0 }
	}

	pub fn pos(&self) -> usize {
		self.cursor
	}

	pub fn len(&self) -> usize {
		self.inner.len()
	}

	pub fn into_inner(self) -> Vec<u8> {
		self.inner
	}

	pub fn inner(&self) -> &[u8] {
		&self.inner
	}

	pub fn inner_mut(&mut self) -> &mut [u8] {
		&mut self.inner
	}

	pub fn set_pos(&mut self, p: usize) {
		if p < self.len() {
			self.cursor = p;
		} else {
			panic!()
		}
	}

	/// Read `l` bytes from `rdr` starting at `pos()`
	pub fn from_reader<R: Read>(
		&mut self,
		rdr: &mut R,
		l: usize,
	) -> Result<usize> {
		let start = self.cursor;
		let end = self.cursor + l;

		if end > self.len() {
			return Err(Error::SizeLimit);
		}

		while end - self.cursor > 0 {
			let i = match rdr.read(&mut self.inner[self.cursor..end])
			{
				Ok(v) => v,
				Err(e) => match e.kind() {
					io::ErrorKind::Interrupted => continue,
					_ => return Err(e.into()),
				},
			};

			if i == 0 {
				return Err(Error::Io(io::Error::from(
					io::ErrorKind::UnexpectedEof,
				)));
			}

			self.cursor += i;
		}

		Ok(self.cursor - start)
	}

	/// Read `l` bytes to `wtr` starting at `pos()`
	pub fn to_writer<W: Write>(
		&mut self,
		wtr: &mut W,
		l: usize,
	) -> Result<usize> {
		let start = self.cursor;
		let end = self.cursor + l;

		if end > self.len() {
			return Err(Error::SizeLimit);
		}

		while end - self.cursor > 0 {
			let i = match wtr.write(&self.inner[self.cursor..end]) {
				Ok(v) => v,
				Err(e) => match e.kind() {
					io::ErrorKind::Interrupted => continue,
					_ => return Err(e.into()),
				},
			};

			if i == 0 {
				return Err(Error::Io(io::Error::from(
					io::ErrorKind::UnexpectedEof,
				)));
			}

			self.cursor += i;
		}

		Ok(self.cursor - start)
	}
}
