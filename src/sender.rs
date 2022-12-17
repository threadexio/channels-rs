use crate::prelude::*;

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`](std::sync::mpsc::Sender),
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
pub struct Sender<T: Serialize, W: Write> {
	_p: PhantomData<T>,
	writer: BufWriter<W>,
	header: packet::Header,
}

impl<T: Serialize, W: Write> Sender<T, W> {
	pub(crate) fn new(writer: W) -> Self {
		Self {
			_p: PhantomData,
			writer: BufWriter::with_capacity(
				packet::MAX_PACKET_SIZE,
				writer,
			),
			header: packet::Header::new(),
		}
	}

	/// Get a handle to the underlying writer.
	pub fn get(&self) -> &W {
		self.writer.get_ref()
	}

	/// Get a handle to the underlying writer. Directly writing to the stream is not advised.
	pub fn get_mut(&mut self) -> &mut W {
		self.writer.get_mut()
	}

	/// Attempts to send an object through the data stream.
	pub fn send(&mut self, data: T) -> Result<()> {
		let raw_data = packet::serialize(&data)?;

		self.header.set_length(raw_data.len().try_into().unwrap());

		self.header.set_id(self.header.get_id().wrapping_add(1));

		// these 2 `write()` calls fill up the entire buffer and we need to flush it
		self.writer.write_all(self.header.finalize())?;
		self.writer.write_all(&raw_data)?;
		self.writer.flush()?;

		Ok(())
	}
}

unsafe impl<T: Serialize, W: Write> Send for Sender<T, W> {}
