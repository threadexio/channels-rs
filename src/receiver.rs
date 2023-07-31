//! Module containing the implementation for [`Receiver`].

use core::marker::PhantomData;

use crate::error::{RecvError, VerifyError};
use crate::io::Reader;
use crate::packet::{consts::*, header::*, Block, LinkedBlocks, Pcb};
use crate::serdes::{self, Deserializer};

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
#[derive(Debug)]
pub struct Receiver<T, R, D> {
	_marker: PhantomData<T>,

	packet: LinkedBlocks,
	pcb: Pcb,

	reader: Reader<R>,
	deserializer: D,
}

#[cfg(feature = "serde")]
impl<T, R> Receiver<T, R, serdes::Bincode> {
	/// Creates a new [`Receiver`] from `reader`.
	pub fn new(reader: R) -> Self {
		Self::with_deserializer(reader, serdes::Bincode)
	}
}

impl<T, R, D> Receiver<T, R, D> {
	/// Create a mew [`Receiver`] from `reader` that uses `deserializer`.
	pub fn with_deserializer(reader: R, deserializer: D) -> Self {
		Self {
			_marker: PhantomData,
			packet: LinkedBlocks::with_payload_capacity(
				MAX_PACKET_SIZE,
			),
			pcb: Pcb::default(),

			reader: Reader::new(reader),
			deserializer,
		}
	}

	/// Get a reference to the underlying reader.
	pub fn get(&self) -> &R {
		self.reader.get()
	}

	/// Get a mutable reference to the underlying reader. Directly
	/// reading from the stream is not advised.
	pub fn get_mut(&mut self) -> &mut R {
		self.reader.get_mut()
	}

	#[cfg(feature = "statistics")]
	/// Get statistics on this [`Receiver`](Self).
	pub fn stats(&self) -> &crate::stats::RecvStats {
		&self.reader.stats
	}
}

/// Read the header from `block` and verify it.
///
/// This function also verifies the `id` field.
fn get_header(
	block: &Block,
	pcb: &mut Pcb,
) -> Result<Header, VerifyError> {
	let header = Header::read_from(block.header())?;

	if header.id != pcb.id {
		return Err(VerifyError::OutOfOrder);
	}

	Ok(header)
}

/// Prepare the receiver to read the next packet.
fn prepare_for_next_packet<R>(reader: &mut Reader<R>, pcb: &mut Pcb) {
	pcb.next();

	#[cfg(feature = "statistics")]
	reader.stats.update_received_time();
}

impl<T, R, D> Receiver<T, R, D>
where
	D: Deserializer<T>,
{
	fn deserialize_packets_to_t(
		&mut self,
	) -> Result<T, RecvError<D::Error>> {
		self.deserializer
			.deserialize(&mut self.packet)
			.map_err(RecvError::Serde)
	}
}

/// An iterator over incoming messages of a [`Receiver`].
///
/// The iterator will return `None` only when [`Receiver::recv_blocking`]
/// returns with an error. When the iterator returns `None` it does not always mean that further
/// calls to `next()` will also return `None`. This behavior depends on the
/// underlying reader.
pub struct Incoming<'r, T, R, D>(&'r mut Receiver<T, R, D>);

mod sync_impl {
	use super::*;

	use std::io::Read;

	impl<T, R, D> Receiver<T, R, D>
	where
		R: Read,
		D: Deserializer<T>,
	{
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

		/// Attempts to receive an object of type `T` from the reader.
		///
		/// This method **will** block until the object has been fully
		/// read.
		///
		/// For the async version of this method, see [`recv`].
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
			self.packet.clear_all();

			let mut i = 0;
			loop {
				if self.packet.blocks.get(i).is_none() {
					self.packet.blocks.push(Block::new());
				}
				let block = &mut self.packet.blocks[i];

				self.reader.read_exact(block.header_mut())?;
				let header = get_header(block, &mut self.pcb)?;

				let payload_len = header.length.to_payload_length();
				self.reader.read_exact(
					&mut block.payload_mut()
						[..payload_len.as_usize()],
				)?;
				block.advance_write(payload_len.as_usize());

				prepare_for_next_packet(
					&mut self.reader,
					&mut self.pcb,
				);

				if !(header.flags & Flags::MORE_DATA) {
					break;
				}

				i += 1;
			}

			self.deserialize_packets_to_t()
		}
	}

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
}

#[cfg(feature = "tokio")]
mod async_tokio_impl {
	use super::*;

	use core::marker::Unpin;

	use tokio::io::{AsyncRead, AsyncReadExt};

	impl<T, R, D> Receiver<T, R, D>
	where
		R: AsyncRead + Unpin,
		D: Deserializer<T>,
	{
		/// Attempts to asynchronously receive an object of type `T`
		/// from the reader.
		///
		/// For the blocking version of this method, see [`recv_blocking`].
		///
		/// # Example
		/// ```no_run
		/// use channels::Receiver;
		///
		/// #[tokio::main]
		/// async fn main() {
		///     let reader = std::io::empty();
		///     let mut reader = Receiver::new(reader);
		///
		///     let number: i32 = reader.recv().await.unwrap();
		/// }
		/// ```
		pub async fn recv(
			&mut self,
		) -> Result<T, RecvError<D::Error>> {
			self.packet.clear_all();

			let mut i = 0;
			loop {
				if self.packet.blocks.get(i).is_none() {
					self.packet.blocks.push(Block::new());
				}
				let block = &mut self.packet.blocks[i];

				self.reader.read_exact(block.header_mut()).await?;
				let header = get_header(block, &mut self.pcb)?;

				let payload_len = header.length.to_payload_length();
				self.reader
					.read_exact(
						&mut block.payload_mut()
							[..payload_len.as_usize()],
					)
					.await?;
				block.advance_write(payload_len.as_usize());

				prepare_for_next_packet(
					&mut self.reader,
					&mut self.pcb,
				);

				if !(header.flags & Flags::MORE_DATA) {
					break;
				}

				i += 1;
			}

			let t = self
				.deserializer
				.deserialize(&mut self.packet)
				.map_err(RecvError::Serde)?;

			Ok(t)
		}
	}
}
