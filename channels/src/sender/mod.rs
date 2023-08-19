//! Module containing the implementation for [`Sender`].

use core::marker::PhantomData;

use crate::error::SendError;
use crate::io::Writer;
use crate::packet::list::List;
use crate::packet::Pcb;
use crate::serdes::*;

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
#[derive(Debug)]
pub struct Sender<T, W, S> {
	_marker: PhantomData<T>,

	packets: List,
	pcb: Pcb,

	writer: Writer<W>,
	serializer: S,
}

cfg_serde! {
	cfg_bincode! {
		impl<T, W> Sender<T, W, Bincode> {
			/// Creates a new [`Sender`] from `writer`.
			pub fn new(writer: W) -> Self {
				Self::with_serializer(writer, Bincode)
			}
		}
	}
}

impl<T, W, S> Sender<T, W, S> {
	/// Create a mew [`Sender`] from `writer` that uses `serializer`.
	pub fn with_serializer(writer: W, serializer: S) -> Self {
		Self {
			_marker: PhantomData,

			packets: List::new(),
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
}

cfg_statistics! {
	impl<T, W,S> Sender<T,W,S> {
		/// Get statistics on this [`Sender`].
		pub fn stats(&self) -> &crate::stats::SendStats {
			&self.writer.stats
		}
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
			.serialize(&mut self.packets, data)
			.map_err(SendError::Serde)
	}
}

cfg_tokio! {
	mod tokio;
}

mod blocking;
