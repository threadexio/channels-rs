use core::marker::Unpin;

use ::tokio::io::{AsyncRead, AsyncReadExt};

use super::*;

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
		self.packets.clear();

		let mut i = 0;
		loop {
			if self.packets.get(i).is_none() {
				self.packets.push(Packet::empty());
			}
			let packet = &mut self.packets[i];

			self.reader.read_exact(packet.header_mut()).await?;
			let header = get_header(packet, &mut self.pcb)?;

			let payload_len = header.length.to_payload_length();
			packet.grow_to(payload_len);
			self.reader
				.read_exact(
					&mut packet.payload_mut()[..payload_len.into()],
				)
				.await?;
			packet.set_write_pos(payload_len);

			prepare_for_next_packet(&mut self.reader, &mut self.pcb);

			if !(header.flags & Flags::MORE_DATA) {
				break;
			}

			i += 1;
		}

		self.deserialize_packets_to_t()
	}
}
