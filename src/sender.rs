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
	///	 - `Err(error)`:	This is a normal `send()` error and should be handled appropriately.
	pub fn send(&mut self, data: T) -> io::Result<()> {
		let mut writer = self.writer.wait_lock();

		let data_size = bincode!()
			.serialized_size(&data)
			.map_err(|x| io::Error::new(io::ErrorKind::Other, x))?;

		let message = Message {
			header: Header {
				payload_len: data_size.try_into().unwrap(),
			},
			payload: data,
		};

		writer.write_all(
			&bincode!()
				.serialize(&message)
				.map_err(|x| io::Error::new(io::ErrorKind::Other, x))?,
		)
	}
}

//unsafe impl<T: Serialize, W: Write> Send for Sender<T, W> {}
