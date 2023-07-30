//! Module containing the implementation for [`Sender`].

use core::borrow::Borrow;
use core::marker::PhantomData;

use crate::error::SendError;
use crate::io::Writer;
use crate::packet::{consts::*, LinkedBlocks, Pcb};
use crate::serdes::{self, Serializer};

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
impl<T, W> Sender<T, W, serdes::Bincode> {
	/// Creates a new [`Sender`] from `writer`.
	pub fn new(writer: W) -> Self {
		Self::with_serializer(writer, serdes::Bincode)
	}
}

impl<T, W, S> Sender<T, W, S> {
	/// Create a mew [`Sender`] from `writer` that uses `serializer`.
	pub fn with_serializer(writer: W, serializer: S) -> Self {
		Self {
			_marker: PhantomData,
			packet: LinkedBlocks::with_payload_capacity(
				MAX_PACKET_SIZE,
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
	fn serialize_t_to_packet(
		&mut self,
		data: &T,
	) -> Result<(), SendError<S::Error>> {
		self.packet.clear_all();

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
		/// Attempts to send an object through the data stream.
		///
		/// # Example
		/// ```no_run
		/// use channels::Sender;
		///
		/// let mut writer = Sender::new(std::io::sink());
		///
		/// writer.send(42_i32).unwrap();
		/// ```
		pub fn send<D>(
			&mut self,
			data: D,
		) -> Result<(), SendError<S::Error>>
		where
			D: Borrow<T>,
		{
			let data = data.borrow();

			self.serialize_t_to_packet(data)?;
			let blocks = self.packet.finalize(&mut self.pcb);

			for block in blocks {
				self.writer.write_all(block.packet())?;
			}

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
		/// TODO: Docs
		pub async fn send_async<D>(
			&mut self,
			data: D,
		) -> Result<(), SendError<S::Error>>
		where
			D: Borrow<T>,
		{
			let data = data.borrow();

			self.serialize_t_to_packet(data)?;
			let blocks = self.packet.finalize(&mut self.pcb);

			for block in blocks {
				self.writer.write_all(block.packet()).await?;
			}

			Ok(())
		}
	}
}
