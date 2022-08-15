use crate::prelude::*;

pub const MAX_MESSAGE_SIZE: u16 = 0xffff;
pub const MESSAGE_HEADER_SIZE: usize = std::mem::size_of::<Header>();

macro_rules! bincode {
	() => {
		bincode::options()
			.reject_trailing_bytes()
			.with_big_endian()
			.with_fixint_encoding()
			.with_limit(crate::common::MAX_MESSAGE_SIZE as u64)
	};
}
pub use bincode;

#[derive(Serialize, Deserialize)]
pub struct Message<T> {
	pub header: Header,
	pub payload: T,
}

#[derive(Serialize, Deserialize)]
pub struct Header {
	pub payload_len: u16,
}

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

pub struct ReadBuffer {
	inner: Box<[u8]>,
	cursor: usize,
}

impl ReadBuffer {
	pub fn with_size(s: usize) -> Self {
		Self {
			inner: vec![0u8; s].into_boxed_slice(),
			cursor: 0,
		}
	}

	pub fn seek(&mut self, i: usize) {
		if i >= self.inner.len() {
			self.cursor = self.inner.len() - 1;
		}

		self.cursor = i;
	}

	pub fn get(&self) -> &[u8] {
		&self.inner
	}

	pub fn read_all<R: Read>(&mut self, r: &mut R, mut l: usize) -> io::Result<usize> {
		let mut total_bytes_read: usize = 0;

		while l != 0 {
			use io::ErrorKind;
			match r.read(&mut self.inner[self.cursor..(self.cursor + l)]) {
				Err(e) => match e.kind() {
					ErrorKind::Interrupted => continue,
					_ => return Err(e),
				},
				Ok(v) => {
					if v == 0 {
						return Err(io::Error::new(ErrorKind::UnexpectedEof, "Unexpected Eof"));
					}

					total_bytes_read += v;
					self.cursor += v;
					l -= v;
				}
			}
		}

		Ok(total_bytes_read)
	}
}
