use crate::prelude::*;

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`](std::sync::mpsc::Receiver),
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Receiver<T: DeserializeOwned, R: Read> {
	_p: PhantomData<T>,
	reader: BufReader<R>,
	header: Option<packet::Header>,
	seq: u16,
}

impl<T: DeserializeOwned, R: Read> Receiver<T, R> {
	pub(crate) fn new(reader: R) -> Self {
		Self {
			_p: PhantomData,
			reader: BufReader::with_capacity(
				packet::MAX_PACKET_SIZE,
				reader,
			),
			header: None,
			seq: 1,
		}
	}

	/// Get a handle to the underlying reader.
	pub fn get(&self) -> &R {
		self.reader.get_ref()
	}

	/// Get a handle to the underlying reader. Directly reading from the stream is not advised.
	pub fn get_mut(&mut self) -> &mut R {
		self.reader.get_mut()
	}

	/// Attempts to read an object from the sender end.
	///
	/// If the underlying data stream is a blocking socket then `recv()` will block until
	/// an object is available.
	///
	/// If the underlying data stream is a non-blocking socket then `recv()` will return
	/// an error with a kind of `std::io::ErrorKind::WouldBlock` whenever the complete object is not
	/// available.
	pub fn recv(&mut self) -> Result<T> {
		if self.header.is_none() {
			let s = self.reader.fill_buf()?;

			if s.is_empty() {
				return Err(Error::Io(
					io::ErrorKind::UnexpectedEof.into(),
				));
			}

			if s.len() < packet::Header::SIZE {
				return Err(Error::Io(
					io::ErrorKind::WouldBlock.into(),
				));
			}

			let header = packet::Header::from_bytes(
				&s[0..packet::Header::SIZE],
			)?;

			if header.get_id() != self.seq {
				return Err(Error::OutOfOrder);
			}
			self.seq = self.seq.wrapping_add(1);

			self.header = Some(header);
			self.reader.consume(packet::Header::SIZE);
		}

		if let Some(ref mut header) = self.header {
			let s = self.reader.fill_buf()?;

			if s.is_empty() {
				return Err(Error::Io(
					io::ErrorKind::UnexpectedEof.into(),
				));
			}

			let data_len: usize = header.get_length().into();

			if s.len() < data_len {
				return Err(Error::Io(
					io::ErrorKind::WouldBlock.into(),
				));
			}

			let data_result =
				packet::deserialize::<T>(&s[..data_len]);

			self.header = None;
			self.reader.consume(data_len);

			return data_result;
		}

		unreachable!()
	}
}

unsafe impl<T: DeserializeOwned, R: Read> Send for Receiver<T, R> {}
