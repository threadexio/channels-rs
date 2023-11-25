//! Module containing the implementation for [`Sender`].

use core::borrow::Borrow;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use alloc::vec::Vec;

use channels_io::prelude::*;
use channels_io::IoSlice;

use channels_packet::{
	slice_to_array_mut, Flags, Header, PayloadLength,
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

		Send::new(self, serialized).await
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

		Send::new(self, serialized)
			.poll_once(|w, buf| w.write_all(buf))
			.unwrap()
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
	W: AsyncWrite,
{
	type Error = W::Error;

	async fn write_all(
		&mut self,
		mut buf: impl Buf,
	) -> Result<(), Self::Error> {
		let l0 = buf.remaining();
		let output = self.inner.write_all(&mut buf).await;
		let l1 = buf.remaining();

		let delta = l0 - l1;
		self.on_write(delta as u64);
		output
	}

	fn flush(
		&mut self,
	) -> impl Future<Output = Result<(), Self::Error>> {
		self.inner.flush()
	}
}

enum State {
	HasPacket { packet: IoSlice<Vec<u8>> },
	NoPacket,
}

struct Send<'a, T, W, S> {
	sender: &'a mut Sender<T, W, S>,
	data: IoSlice<Vec<u8>>,
	has_sent_at_least_one: bool,
	state: State,
}

impl<'a, T, W, S> Send<'a, T, W, S>
where
	S: Serializer<T>,
{
	fn new(
		sender: &'a mut Sender<T, W, S>,
		data: IoSlice<Vec<u8>>,
	) -> Self {
		Self {
			sender,
			data,
			has_sent_at_least_one: false,
			state: State::NoPacket,
		}
	}

	fn poll_once<F, E>(
		&mut self,
		mut write_all: F,
	) -> Poll<Result<(), SendError<S::Error, E>>>
	where
		F: FnMut(
			&mut StatWriter<W>,
			&mut dyn Buf,
		) -> Poll<Result<(), E>>,
	{
		use core::cmp::min;
		use Poll::Ready;

		loop {
			match self.state {
				State::NoPacket => {
					let payload_length = unsafe {
						let saturated = min(
							self.data.remaining(),
							PayloadLength::MAX.as_usize(),
						);

						PayloadLength::new_unchecked(saturated as u16)
					};

					if payload_length.as_usize() == 0
						&& self.has_sent_at_least_one
					{
						return Ready(Ok(()));
					}

					let packet_length =
						payload_length.to_packet_length();

					let mut packet =
						vec![0u8; packet_length.as_usize()];

					if payload_length.as_usize() != 0 {
						let n = channels_io::copy_slice(
							self.data.unfilled(),
							&mut packet[Header::SIZE..],
						);
						self.data.advance(n);

						if n < payload_length.as_usize() {
							packet.resize(Header::SIZE + n, 0);
						}
					}

					Header {
						length: packet_length,
						flags: Flags::zero()
							.set_if(Flags::MORE_DATA, |_| {
								self.data.has_remaining()
							}),
						id: self.sender.pcb.id_gen.next_id(),
					}
					.write_to(unsafe {
						// SAFETY: The range guarantees that the slice is exactly equal to
						//         Header::SIZE.
						slice_to_array_mut(
							&mut packet[..Header::SIZE],
						)
					});

					self.has_sent_at_least_one = true;

					self.state = State::HasPacket {
						packet: IoSlice::new(packet),
					};
				},
				State::HasPacket { ref mut packet } => {
					match ready!(write_all(
						&mut self.sender.writer,
						packet
					)) {
						Err(e) => {
							return Ready(Err(SendError::Io(e)))
						},
						Ok(_) if !packet.has_remaining() => {
							self.state = State::NoPacket;
						},
						Ok(_) => {},
					}
				},
			}
		}
	}
}

impl<'a, T, W, S> Future for Send<'a, T, W, S>
where
	W: AsyncWrite + Unpin,
	S: Serializer<T>,
{
	type Output = Result<(), SendError<S::Error, W::Error>>;

	fn poll(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		self.get_mut().poll_once(|w, buf| {
			let mut fut = w.write_all(buf);
			// SAFETY: `fut` will never be moved because it is allocated in this
			//         clojure and is never moved inside any function.
			unsafe { Pin::new_unchecked(&mut fut) }.poll(cx)
		})
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
