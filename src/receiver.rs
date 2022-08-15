use crate::prelude::*;

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`](std::sync::mpsc::Receiver),
/// except for a [few key differences](self).
///
/// See [module-level documentation](self).
pub struct Receiver<T: DeserializeOwned, R: Read> {
	_p: PhantomData<T>,

	reader: Arc<Inner<R>>,

	recv_buf: ReadBuffer,
	msg_header: Option<Header>,
}

impl<T: DeserializeOwned, R: Read> Receiver<T, R> {
	pub(crate) fn new(reader: Arc<Inner<R>>) -> Self {
		Self {
			_p: PhantomData,
			recv_buf: ReadBuffer::with_size(MAX_MESSAGE_SIZE as usize),
			msg_header: None,
			reader,
		}
	}

	/// Get a handle to the underlying stream
	pub fn inner(&self) -> MutexGuard<'_, R> {
		self.reader.wait_lock()
	}

	/// Attempts to read an object from the sender end.
	///
	/// If the underlying data stream is a blocking socket then `recv()` will block until
	/// an object is available.
	///
	/// If the underlying data stream is a non-blocking socket then `recv()` will return
	/// an error with a kind of `std::io::ErrorKind::WouldBlock` whenever the complete object is not
	/// available.
	///
	/// The method returns as follows:
	///  - `Ok(object)`:	The receive operation was successful and an object was returned.
	///  - `Err(error)`:	If `error.kind()` is `std::io::ErrorKind::WouldBlock` then no object
	/// 					is currently available, but one might become available in the future
	/// 					(This can only happen when the underlying stream is set to non-blocking mode).
	///	 - `Err(error)`:	This is a normal `read()` error and should be handled appropriately.
	pub fn recv(&mut self) -> io::Result<T> {
		let mut reader = self.reader.wait_lock();

		if self.msg_header.is_none() {
			self.recv_buf.read_all(&mut *reader, MESSAGE_HEADER_SIZE)?;

			self.recv_buf.seek(0);
			self.msg_header = Some(
				bincode!()
					.deserialize(&self.recv_buf.get()[..MESSAGE_HEADER_SIZE])
					.map_err(|x| io::Error::new(io::ErrorKind::Other, x))?,
			);
		}

		if let Some(header) = &self.msg_header {
			self.recv_buf
				.read_all(&mut *reader, header.payload_len as usize)?;

			let data = bincode!()
				.deserialize(&self.recv_buf.get()[..header.payload_len as usize])
				.map_err(|x| io::Error::new(io::ErrorKind::Other, x))?;

			self.recv_buf.seek(0);
			self.msg_header = None;

			return Ok(data);
		}

		unreachable!()

		/*
		// check if we haven't read a message header yet
		if self.msg_header.is_none() {
			// continuously read to complete the header, if any error is encountered return immediately
			// when working with non-blocking sockets this code returns WouldBlock if there is no data,
			// this is the desired behavior
			while self.recv_cursor != MESSAGE_HEADER_SIZE {
				match reader.read(&mut self.recv_buf[self.recv_cursor..MESSAGE_HEADER_SIZE]) {
					Ok(v) => self.recv_cursor += v,
					Err(e) => match e.kind() {
						io::ErrorKind::Interrupted => continue,
						_ => return Err(e),
					},
				};
			}

			self.recv_cursor = 0;
			self.msg_header = Some(
				bincode!()
					.deserialize(&self.recv_buf[..MESSAGE_HEADER_SIZE])
					.map_err(|x| io::Error::new(io::ErrorKind::Other, x))?,
			);
		}

		if let Some(ref hdr) = self.msg_header {
			while self.recv_cursor != hdr.payload_len as usize {
				match reader.read(&mut self.recv_buf[self.recv_cursor..hdr.payload_len as usize]) {
					Ok(v) => self.recv_cursor += v,
					Err(e) => match e.kind() {
						io::ErrorKind::Interrupted => continue,
						_ => return Err(e),
					},
				};
			}

			let data = bincode!()
				.deserialize(&self.recv_buf[..hdr.payload_len as usize])
				.map_err(|x| io::Error::new(io::ErrorKind::Other, x))?;

			self.recv_cursor = 0;
			self.msg_header = None;

			return Ok(data);
		}

		return Err(io::Error::new(
			io::ErrorKind::WouldBlock,
			"failed to fill buffer",
		));
		*/
	}
}

//unsafe impl<T: DeserializeOwned, R: Read> Send for Receiver<T, R> {}
