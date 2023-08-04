use std::io::Read;

use super::Receiver;
use super::{get_header, prepare_for_next_packet};
use crate::error::RecvError;
use crate::packet::*;
use crate::serdes::Deserializer;

impl<T, R, D> Receiver<T, R, D>
where
	R: Read,
	D: Deserializer<T>,
{
	/// Attempts to receive an object of type `T` from the reader.
	///
	/// This method **will** block until the object has been fully
	/// read.
	///
	#[cfg_attr(
		feature = "tokio",
		doc = "For the async version of this method, see [`Receiver::recv`]."
	)]
	///
	/// # Example
	/// ```no_run
	/// use channels::Receiver;
	///
	/// fn main() {
	///     let reader = std::io::empty();
	///     let mut reader = Receiver::new(reader);
	///
	///     let number: i32 = reader.recv_blocking().unwrap();
	/// }
	/// ```
	pub fn recv_blocking(
		&mut self,
	) -> Result<T, RecvError<D::Error>> {
		self.packet.clear_payload();

		let mut i = 0;
		loop {
			if self.packet.blocks.get(i).is_none() {
				self.packet.blocks.push(Block::new());
			}
			let block = &mut self.packet.blocks[i];

			self.reader.read_exact(block.header_mut())?;
			let header = get_header(block, &mut self.pcb)?;

			let payload_len = header.length.to_payload_length();
			block.grow_payload_to(payload_len);
			self.reader.read_exact(
				&mut block.payload_mut()[..payload_len.as_usize()],
			)?;
			block.advance_write(payload_len.as_usize());

			prepare_for_next_packet(&mut self.reader, &mut self.pcb);

			if !(header.flags & Flags::MORE_DATA) {
				break;
			}

			i += 1;
		}

		self.deserialize_packets_to_t()
	}

	/// Get an iterator over incoming messages. The iterator will
	/// return `None` messages when an error is returned by [`Receiver::recv_blocking`].
	///
	/// See: [`Incoming`].
	///
	/// # Example
	/// ```no_run
	/// use channels::Receiver;
	///
	/// fn main() {
	///     let reader = std::io::empty();
	///     let mut rx = Receiver::<i32, _, _>::new(reader);
	///
	///     for number in rx.incoming() {
	///         println!("Received number: {number}");
	///     }
	/// }
	/// ```
	pub fn incoming(&mut self) -> Incoming<T, R, D> {
		Incoming(self)
	}
}

/// An iterator over incoming messages of a [`Receiver`].
///
/// The iterator will return `None` only when [`Receiver::recv_blocking`]
/// returns with an error. When the iterator returns `None` it does not always mean that further
/// calls to `next()` will also return `None`. This behavior depends on the
/// underlying reader.
pub struct Incoming<'r, T, R, D>(&'r mut Receiver<T, R, D>);

impl<T, R, D> Iterator for Incoming<'_, T, R, D>
where
	R: Read,
	D: Deserializer<T>,
{
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.recv_blocking().ok()
	}
}