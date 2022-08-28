use crate::prelude::*;

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`](std::sync::mpsc::Sender),
/// except for a [few key differences](self).
///
/// See [module-level documentation](self).
pub struct Sender<T: Serialize, W: Write> {
	_p: PhantomData<T>,

	writer: Arc<Inner<W>>,
}

impl<T: Serialize, W: Write> Sender<T, W> {
	pub(crate) fn new(writer: Arc<Inner<W>>) -> Self {
		Self {
			_p: PhantomData,
			writer,
		}
	}

	/// Get a handle to the underlying stream
	pub fn inner(&self) -> MutexGuard<'_, W> {
		self.writer.wait_lock()
	}

	/// Attempts to send an object through the data stream.
	///
	/// The method returns as follows:
	///  - `Ok(())`:		The send operation was successful and the object was sent.
	///	 - `Err(error)`:	This is a normal `write()` error and should be handled appropriately.
	pub fn send(&mut self, data: T) -> Result<()> {
		let data_length = serialized_size(&data)?;

		if data_length > MAX_PAYLOAD_SIZE.into() {
			return Err(Error::DataTooLarge);
		}

		let data = serialize(&data)?;

		let header = serialize(&Header {
			protocol_version: PROTOCOL_VERSION,
			payload_len: data_length as u16,

			#[cfg(not(feature = "crc"))]
			payload_checksum: 0,

			#[cfg(feature = "crc")]
			payload_checksum: crate::crc::checksum32(&data),
		})?;

		let mut writer = self.writer.wait_lock();

		writer.write_all(&[header, data].concat())?;

		Ok(())
	}
}

unsafe impl<T: Serialize, W: Write> Send for Sender<T, W> {}
