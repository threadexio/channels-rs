//! Module containing the implementation for [`Sender`].

use core::borrow::Borrow;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use alloc::vec::Vec;

use channels_io::prelude::*;
use channels_io::IoSlice;

use channels_packet::{
	slice_to_array_mut, Flags, Header, IdGenerator, PayloadLength,
};
use channels_serdes::Serializer;

#[allow(unused_imports)]
use crate::common::{Pcb, Statistics};
use crate::error::SendError;

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
#[derive(Debug, Clone)]
pub struct Sender<T, W, S> {
	_marker: PhantomData<T>,
	writer: StatWriter<W>,
	serializer: S,
	pcb: Pcb,
}

impl<T> Sender<T, (), ()> {
	/// Create a new builder.
	pub const fn builder() -> Builder<T, (), ()> {
		Builder::new()
	}
}

impl<T, W, S> Sender<T, W, S> {
	/// Get a reference to the underlying writer.
	pub fn get(&self) -> &W {
		&self.writer.inner
	}

	/// Get a mutable reference to the underlying writer. Directly writing to
	/// the stream is not advised.
	pub fn get_mut(&mut self) -> &mut W {
		&mut self.writer.inner
	}

	/// Get statistics on this sender.
	#[cfg(feature = "statistics")]
	pub fn statistics(&self) -> &Statistics {
		&self.writer.statistics
	}
}

impl<T, W, S> Sender<T, W, S>
where
	W: AsyncWrite + Unpin,
	S: Serializer<T>,
{
	/// Attempts to send `data` through the channel.
	///
	/// This function will return a future that will complete only when all the
	/// bytes of `data` have been sent through the channel.
	pub async fn send<D>(
		&mut self,
		data: D,
	) -> Result<(), SendError<S::Error, W::Error>>
	where
		D: Borrow<T>,
	{
		let serialized =
			serialize_type(&mut self.serializer, data.borrow())
				.map_err(SendError::Serde)?;

		let mut packets =
			PacketIter::new(&mut self.pcb.id_gen, serialized);

		while let Some(packet) = packets.next_packet() {
			self.writer
				.write_all(packet)
				.await
				.map_err(SendError::Io)?;
		}

		self.writer.flush().await.map_err(SendError::Io)?;

		Ok(())
	}
}

impl<T, W, S> Sender<T, W, S>
where
	W: Write,
	S: Serializer<T>,
{
	/// Attempts to send `data` through the channel.
	///
	/// This function will block the current thread until every last byte of
	/// `data` has been sent.
	///
	/// # Panics
	///
	/// Panics if the underlying writer returns with `WouldBlock`.
	#[track_caller]
	pub fn send_blocking<D>(
		&mut self,
		data: D,
	) -> Result<(), SendError<S::Error, W::Error>>
	where
		D: Borrow<T>,
	{
		let serialized =
			serialize_type(&mut self.serializer, data.borrow())
				.map_err(SendError::Serde)?;

		let mut packets =
			PacketIter::new(&mut self.pcb.id_gen, serialized);

		while let Some(packet) = packets.next_packet() {
			self.writer
				.write_all(packet)
				.unwrap()
				.map_err(SendError::Io)?;
		}

		self.writer.flush().unwrap().map_err(SendError::Io)?;

		Ok(())
	}
}

/// A builder that when completed will return a [`Sender`].
#[derive(Debug)]
pub struct Builder<T, W, S> {
	_marker: PhantomData<T>,
	writer: W,
	serializer: S,
}

impl<T> Builder<T, (), ()> {
	/// Create a new [`Builder`] with the default options.
	pub const fn new() -> Self {
		Builder { _marker: PhantomData, serializer: (), writer: () }
	}
}

impl<T> Default for Builder<T, (), ()> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T, S> Builder<T, (), S> {
	/// Use this synchronous reader.
	pub fn writer<W: Write>(
		self,
		writer: impl IntoWriter<W>,
	) -> Builder<T, W, S> {
		Builder {
			_marker: PhantomData,
			writer: writer.into_writer(),
			serializer: self.serializer,
		}
	}

	/// Use this asynchronous reader.
	pub fn async_writer<W: AsyncWrite>(
		self,
		writer: impl IntoAsyncWriter<W>,
	) -> Builder<T, W, S> {
		Builder {
			_marker: PhantomData,
			writer: writer.into_async_writer(),
			serializer: self.serializer,
		}
	}
}

impl<T, W> Builder<T, W, ()> {
	/// Use this serializer.
	pub fn serializer<S>(self, serializer: S) -> Builder<T, W, S>
	where
		S: Serializer<T>,
	{
		Builder {
			_marker: PhantomData,
			writer: self.writer,
			serializer,
		}
	}
}

impl<T, W, S> Builder<T, W, S> {
	/// Finalize the builder and build a [`Sender`].
	pub fn build(self) -> Sender<T, W, S> {
		Sender {
			_marker: PhantomData,
			writer: StatWriter::new(self.writer),
			serializer: self.serializer,
			pcb: Pcb::new(),
		}
	}
}

#[derive(Debug, Clone)]
struct StatWriter<W> {
	inner: W,

	#[cfg(feature = "statistics")]
	statistics: Statistics,
}

impl<W> StatWriter<W> {
	pub const fn new(writer: W) -> Self {
		Self {
			inner: writer,

			#[cfg(feature = "statistics")]
			statistics: Statistics::new(),
		}
	}

	#[allow(unused_variables)]
	fn on_write(&mut self, n: u64) {
		#[cfg(feature = "statistics")]
		self.statistics.add_total_bytes(n);
	}
}

impl<W> Write for StatWriter<W>
where
	W: Write,
{
	type Error = W::Error;
	#[track_caller]

	fn write_all(
		&mut self,
		mut buf: impl Buf,
	) -> Poll<Result<(), Self::Error>> {
		let l0 = buf.remaining();
		let output = self.inner.write_all(&mut buf);
		let l1 = buf.remaining();

		let delta = l0 - l1;
		self.on_write(delta as u64);
		output
	}

	fn flush(&mut self) -> Poll<Result<(), Self::Error>> {
		self.inner.flush()
	}
}

impl<W> AsyncWrite for StatWriter<W>
where
	W: AsyncWrite + Unpin,
{
	type Error = W::Error;

	fn poll_write_all(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		mut buf: impl Buf,
	) -> Poll<Result<(), Self::Error>> {
		let l0 = buf.remaining();
		let output =
			Pin::new(&mut self.inner).poll_write_all(cx, &mut buf);
		let l1 = buf.remaining();

		let delta = l0 - l1;
		self.on_write(delta as u64);
		output
	}

	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		Pin::new(&mut self.inner).poll_flush(cx)
	}
}

struct PacketIter<'a> {
	id_gen: &'a mut IdGenerator,
	serialized: IoSlice<Vec<u8>>,
	yielded_at_least_once: bool,
	packet: Vec<u8>,
}

impl<'a> PacketIter<'a> {
	fn new(
		id_gen: &'a mut IdGenerator,
		serialized: IoSlice<Vec<u8>>,
	) -> Self {
		Self {
			id_gen,
			packet: Vec::new(),
			serialized,
			yielded_at_least_once: false,
		}
	}

	fn prepare_packet(&mut self) -> &[u8] {
		use core::cmp::min;

		let payload_length = unsafe {
			let saturated = min(
				self.serialized.remaining(),
				PayloadLength::MAX.as_usize(),
			);

			PayloadLength::new_unchecked(saturated as u16)
		};

		let packet_length = payload_length.to_packet_length();

		self.packet.resize(packet_length.as_usize(), 0);

		if payload_length.as_usize() != 0 {
			let n = channels_io::copy_slice(
				self.serialized.unfilled(),
				&mut self.packet[Header::SIZE..],
			);
			self.serialized.advance(n);

			if n < payload_length.as_usize() {
				self.packet.resize(Header::SIZE + n, 0);
			}
		}

		Header {
			length: packet_length,
			flags: Flags::zero().set_if(Flags::MORE_DATA, |_| {
				self.serialized.has_remaining()
			}),
			id: self.id_gen.next_id(),
		}
		.write_to(unsafe {
			// SAFETY: The range guarantees that the slice is exactly equal to
			//         Header::SIZE.
			slice_to_array_mut(&mut self.packet[..Header::SIZE])
		});

		&self.packet
	}

	fn next_packet(&mut self) -> Option<&[u8]> {
		// Why `yielded_at_least_once` is needed:
		//   Senders must always send one packet regardless of whether or not
		//   the serialized type takes up any space. In case a type serializes
		//   to zero bytes, the sender must send exactly one empty packet.
		if !self.yielded_at_least_once {
			self.yielded_at_least_once = true;
			let packet = self.prepare_packet();
			Some(packet)
		} else if self.serialized.has_remaining() {
			let packet = self.prepare_packet();
			Some(packet)
		} else {
			None
		}
	}
}

fn serialize_type<T, S>(
	serializer: &mut S,
	t: &T,
) -> Result<IoSlice<Vec<u8>>, S::Error>
where
	S: Serializer<T>,
{
	serializer.serialize(t).map(IoSlice::new)
}
