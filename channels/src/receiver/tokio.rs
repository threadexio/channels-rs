use core::marker::Unpin;

use ::tokio::io::{AsyncRead, AsyncReadExt};

use super::{get_header, Receiver};
use crate::error::RecvError;
use crate::packet::list::Packet;
use crate::packet::types::Flags;
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

			if !(header.flags & Flags::MORE_DATA) {
				break;
			}

			i += 1;
		}

		#[cfg(feature = "statistics")]
		self.reader.stats.update_received_time();

		self.deserialize_packets_to_t()
	}

	/// Attempts to receive an object of type `T` from the underlying
	/// asynchronous reader.
	///
	/// It is not to be confused with [`Receiver::recv_blocking`].
	/// This method is only available for asynchronous readers and its
	/// only purpose is to serve a bridge between asynchronous and
	/// synchronous code. That said, it is almost always preferable to
	/// use directly the async API and `.await` where necessary.
	///
	/// You can call this method from inside an asynchronous runtime,
	/// but please note that it **will** block the entire runtime. In
	/// other words, any other tasks will not run until this completes.
	/// For this reason, it is not advised to use this in an asynchronous
	/// context.
	///
	/// # Example
	/// ```no_run
	/// use channels::Receiver;
	///
	/// fn main() {
	///     let reader = tokio::io::empty();
	///     let mut reader = Receiver::new(reader);
	///
	///     let number: i32 = reader.blocking_recv().unwrap();
	/// }
	/// ```
	pub fn blocking_recv(
		&mut self,
	) -> Result<T, RecvError<D::Error>> {
		crate::util::block_on(async { self.recv().await })?
	}
}
