use core::marker::Unpin;

use tokio::io::{AsyncRead, AsyncReadExt};

use super::{get_header, prepare_for_next_packet, Receiver};
use crate::error::RecvError;
use crate::packet::*;
use crate::serdes::Deserializer;

impl<T, R, D> Receiver<T, R, D>
where
	R: AsyncRead + Unpin,
	D: Deserializer<T>,
{
	/// Attempts to asynchronously receive an object of type `T`
	/// from the reader.
	///
	/// For the blocking version of this method, see [`Receiver::recv_blocking`].
	///
	/// # Example
	/// ```no_run
	/// use channels::Receiver;
	///
	/// #[tokio::main]
	/// async fn main() {
	///     let reader = tokio::io::empty();
	///     let mut reader = Receiver::new(reader);
	///
	///     let number: i32 = reader.recv().await.unwrap();
	/// }
	/// ```
	pub async fn recv(&mut self) -> Result<T, RecvError<D::Error>> {
		self.packet.clear_payload();

		let mut i = 0;
		loop {
			if self.packet.blocks.get(i).is_none() {
				self.packet.blocks.push(Block::new());
			}
			let block = &mut self.packet.blocks[i];

			self.reader.read_exact(block.header_mut()).await?;
			let header = get_header(block, &mut self.pcb)?;

			let payload_len = header.length.to_payload_length();
			block.grow_payload_to(payload_len);
			self.reader
				.read_exact(
					&mut block.payload_mut()[..payload_len.into()],
				)
				.await?;
			block.advance_write(payload_len.into());

			prepare_for_next_packet(&mut self.reader, &mut self.pcb);

			if !(header.flags & Flags::MORE_DATA) {
				break;
			}

			i += 1;
		}

		self.deserialize_packets_to_t()
	}
}
