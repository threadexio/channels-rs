use core::borrow::Borrow;
use core::marker::Unpin;

use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::Sender;
use crate::error::SendError;
use crate::serdes::Serializer;

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
		self.packet.clear_payload();

		self.serialize_t_to_packets(data)?;
		let blocks = self.packet.finalize(&mut self.pcb);

		for block in blocks {
			self.writer.write_all(block.packet()).await?;
		}

		#[cfg(feature = "statistics")]
		self.writer.stats.update_sent_time();

		Ok(())
	}
}
