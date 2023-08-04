use core::borrow::Borrow;

use std::io::Write;

use super::Sender;
use crate::error::SendError;
use crate::serdes::Serializer;

impl<T, W, S> Sender<T, W, S>
where
	W: Write,
	S: Serializer<T>,
{
	/// Attempts to send an object of type `T` through the writer.
	///
	/// This method **will** block until the `data` has been fully
	/// sent.
	///
	#[cfg_attr(
		feature = "tokio",
		doc = "For the async version of this method, see [`Sender::send`]."
	)]
	///
	/// # Example
	/// ```no_run
	/// use channels::Sender;
	///
	/// fn main() {
	///     let writer = std::io::sink();
	///     let mut tx = Sender::new(writer);
	///
	///     tx.send_blocking(42_i32).unwrap();
	/// }
	/// ```
	pub fn send_blocking<D>(
		&mut self,
		data: D,
	) -> Result<(), SendError<S::Error>>
	where
		D: Borrow<T>,
	{
		let data = data.borrow();
		self.packet.clear_payload();

		self.serialize_t_to_packets(data)?;
		let blocks = self.packet.finalize(&mut self.pcb);

		for block in blocks {
			self.writer.write_all(block.packet())?;
		}

		#[cfg(feature = "statistics")]
		self.writer.stats.update_sent_time();

		Ok(())
	}
}
