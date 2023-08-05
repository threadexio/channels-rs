use core::borrow::Borrow;
use core::marker::Unpin;

use ::tokio::io::{AsyncWrite, AsyncWriteExt};

use super::*;

impl<T, W, S> Sender<T, W, S>
where
	W: AsyncWrite + Unpin,
	S: Serializer<T>,
{
	/// Attempts to asynchronously send an object of type `T`
	/// through the writer.
	///
	/// For the blocking version of this method, see [`Sender::send_blocking`].
	///
	/// # Example
	/// ```no_run
	/// use channels::Sender;
	///
	/// #[tokio::main]
	/// async fn main() {
	///     let writer = tokio::io::sink();
	///     let mut sender = Sender::<i32, _, _>::new(writer);
	///
	///     sender.send(42_i32).await.unwrap();
	/// }
	/// ```
	pub async fn send<D>(
		&mut self,
		data: D,
	) -> Result<(), SendError<S::Error>>
	where
		D: Borrow<T>,
	{
		let data = data.borrow();
		self.packets.clear();

		self.serialize_t_to_packets(data)?;
		let packets =
			finalize_packets(&mut self.packets, &mut self.pcb);
		for packet in packets {
			self.writer.write_all(packet.initialized()).await?;
		}

		#[cfg(feature = "statistics")]
		self.writer.stats.update_sent_time();

		Ok(())
	}
}
