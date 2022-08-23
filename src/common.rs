use crate::prelude::*;

pub struct Inner<T> {
	lock: Mutex<T>,
}

impl<T> Inner<T> {
	pub fn new(stream: T) -> Self {
		Self {
			lock: Mutex::new(stream),
		}
	}

	pub fn wait_lock(&self) -> MutexGuard<'_, T> {
		self.lock.lock().unwrap()
	}
}

pub struct Buffer {
	inner: Vec<u8>,
	cursor: usize,
}

#[allow(dead_code)]
impl Buffer {
	pub fn with_size(s: usize) -> Self {
		Self {
			inner: vec![0u8; s],
			cursor: 0,
		}
	}

	pub fn pos(&self) -> usize {
		self.cursor
	}

	pub fn set_pos(&mut self, p: usize) -> Result<()> {
		if p >= self.capacity() {
			Err(error!(ChannelError::ObjectTooLarge))
		} else {
			self.cursor = p;
			Ok(())
		}
	}

	pub fn buffer(&self) -> &[u8] {
		&self.inner
	}

	pub fn capacity(&self) -> usize {
		self.inner.len()
	}

	pub fn len(&self) -> usize {
		self.cursor
	}

	/// Copy `s` starting at `pos()`
	pub fn append_slice(&mut self, s: &[u8]) -> Result<()> {
		if self.cursor + s.len() >= self.capacity() {
			return Err(error!(ChannelError::ObjectTooLarge));
		}

		for i in 0..s.len() {
			self.inner[self.cursor + i] = s[i];
		}

		Ok(())
	}

	/// Consume `l` bytes from `pos()`
	pub fn consume(&mut self, l: usize) -> Result<&[u8]> {
		if self.cursor + l >= self.capacity() {
			return Err(error!(ChannelError::ObjectTooLarge));
		}

		Ok(&self.inner[self.cursor..self.cursor + l])
	}

	/// Read `l` bytes from `rdr` starting at `pos()`
	pub fn from_reader<R: Read>(&mut self, rdr: &mut R, l: usize) -> Result<usize> {
		let start = self.cursor;
		let end = self.cursor + l;

		if end >= self.capacity() {
			return Err(error!(ChannelError::ObjectTooLarge));
		}

		while end - self.cursor > 0 {
			let i = rdr.read(&mut self.inner[self.cursor..end])?;

			if i == 0 {
				return Err(Error::new(
					ErrorKind::UnexpectedEof,
					ErrorKind::UnexpectedEof.to_string(),
				));
			}

			self.cursor += i;
		}

		Ok(self.cursor - start)
	}

	/// Read `l` bytes to `wtr` starting at `pos()`
	pub fn to_writer<W: Write>(&mut self, wtr: &mut W, l: usize) -> Result<usize> {
		let start = self.cursor;
		let end = self.cursor + l;

		if end >= self.capacity() {
			return Err(error!(ChannelError::ObjectTooLarge));
		}

		while end - self.cursor > 0 {
			let i = wtr.write(&self.inner[self.cursor..end])?;

			if i == 0 {
				return Err(Error::new(
					ErrorKind::UnexpectedEof,
					ErrorKind::UnexpectedEof.to_string(),
				));
			}

			self.cursor += i;
		}

		Ok(self.cursor - start)
	}
}
