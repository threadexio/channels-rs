//! Module containing the implementation for [`Sender`].

use core::borrow::Borrow;
use core::marker::PhantomData;

use crate::error::SendError;
use crate::io::Writer;
use crate::packet::{consts::*, LinkedBlocks, Pcb};
use crate::serdes::*;

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
#[derive(Debug)]
pub struct Sender<T, W, S> {
	_marker: PhantomData<T>,

	packet: LinkedBlocks,
	pcb: Pcb,

	writer: Writer<W>,
	serializer: S,
}

#[cfg(feature = "serde")]
impl<T, W> Sender<T, W, Bincode> {
	/// Creates a new [`Sender`] from `writer`.
	pub fn new(writer: W) -> Self {
		Self::with_serializer(writer, Bincode)
	}
}

impl<T, W, S> Sender<T, W, S> {
	/// Create a mew [`Sender`] from `writer` that uses `serializer`.
	pub fn with_serializer(writer: W, serializer: S) -> Self {
		Self {
			_marker: PhantomData,
			packet: LinkedBlocks::with_total_payload_capacity(
				MAX_PAYLOAD_SIZE,
			),
			pcb: Pcb::default(),

			writer: Writer::new(writer),
			serializer,
		}
	}

	/// Get a reference to the underlying reader.
	pub fn get(&self) -> &W {
		self.writer.get()
	}

	/// Get a mutable reference to the underlying reader. Directly reading from the stream is not advised.
	pub fn get_mut(&mut self) -> &mut W {
		self.writer.get_mut()
	}

	#[cfg(feature = "statistics")]
	/// Get statistics on this [`Sender`].
	pub fn stats(&self) -> &crate::stats::SendStats {
		&self.writer.stats
	}
}

impl<T, W, S> Sender<T, W, S>
where
	S: Serializer<T>,
{
	fn serialize_t_to_packets(
		&mut self,
		data: &T,
	) -> Result<(), SendError<S::Error>> {
		self.serializer
			.serialize(&mut self.packet, data)
			.map_err(SendError::Serde)
	}
}

mod sync_impl {
	use super::*;

	use std::io::Write;

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
		/// For the async version of this method, see [`send`].
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
}

#[cfg(feature = "tokio")]
mod async_tokio_impl {
	use super::*;

	use core::marker::Unpin;

	use tokio::io::{AsyncWrite, AsyncWriteExt};

	impl<T, W, S> Sender<T, W, S>
	where
		W: AsyncWrite + Unpin,
		S: Serializer<T>,
	{
		/// Attempts to asynchronously send an object of type `T`
		/// through the writer.
		///
		/// For the blocking version of this method, see [`send_blocking`].
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
}
