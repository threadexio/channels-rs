//! Module containing the implementation for [`Receiver`].

use core::marker::PhantomData;

use crate::error::{RecvError, VerifyError};
use crate::io::Reader;
use crate::macros::*;
use crate::packet::{consts::*, header::*, Block, LinkedBlocks, Pcb};
use crate::serdes::*;

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

cfg_serde! {
	impl<T, R> Receiver<T, R, Bincode> {
		/// Creates a new [`Receiver`] from `reader`.
		pub fn new(reader: R) -> Self {
			Self::with_deserializer(reader, Bincode)
		}
	}
}

impl<T, R, D> Receiver<T, R, D> {
	/// Create a mew [`Receiver`] from `reader` that uses `deserializer`.
	pub fn with_deserializer(reader: R, deserializer: D) -> Self {
		Self {
			_marker: PhantomData,
			packet: LinkedBlocks::with_total_payload_capacity(
				MAX_PAYLOAD_SIZE,
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
}

cfg_statistics! {
	impl<T, R, D> Receiver<T, R, D> {
		/// Get statistics on this [`Receiver`](Self).
		pub fn stats(&self) -> &crate::stats::RecvStats {
			&self.reader.stats
		}
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
#[allow(unused_variables)]
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

cfg_tokio! {
	mod tokio;
}

mod blocking;

pub use blocking::Incoming;
