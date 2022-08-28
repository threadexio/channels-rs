use crate::prelude::*;

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`](std::sync::mpsc::Receiver),
/// except for a [few key differences](self).
///
/// See [module-level documentation](self).
pub struct Receiver<T: DeserializeOwned, R: Read> {
	_p: PhantomData<T>,

	reader: Arc<Inner<R>>,

	recv_buf: Buffer,
	msg_header: Option<Header>,
}

impl<T: DeserializeOwned, R: Read> Receiver<T, R> {
	pub(crate) fn new(reader: Arc<Inner<R>>) -> Self {
		Self {
			_p: PhantomData,
			recv_buf: Buffer::with_size(MAX_PAYLOAD_SIZE as usize),
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
	pub fn recv(&mut self) -> Result<T> {
		let mut reader = self.reader.wait_lock();

		if self.msg_header.is_none() {
			self.recv_buf.from_reader(&mut *reader, HEADER_SIZE)?;

			let header: Header = deserialize(&self.recv_buf.buffer()[..HEADER_SIZE])?;

			// This is here to reset the state in case we don't pass the checks bellow.
			self.recv_buf
				.set_pos(0)
				.expect("set_pos(0) failed. it should not have failed");

			if header.protocol_version != PROTOCOL_VERSION {
				return Err(Error::VersionMismatch);
			}

			self.msg_header = Some(header);
		}

		if let Some(header) = &self.msg_header {
			self.recv_buf
				.from_reader(&mut *reader, header.payload_len as usize)?;

			let serialized_data = &self.recv_buf.buffer()[..header.payload_len as usize];

			#[cfg(feature = "crc")]
			if crate::crc::checksum32(&serialized_data) != header.payload_checksum {
				return Err(Error::ChecksumError);
			}

			let data = deserialize(serialized_data)?;

			// reset state
			self.recv_buf.set_pos(0)?;
			self.msg_header = None;

			return Ok(data);
		}

		unreachable!()
	}
}

unsafe impl<T: DeserializeOwned, R: Read> Send for Receiver<T, R> {}
