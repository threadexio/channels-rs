//! Module containing the implementation for [`Sender`].

use core::borrow::Borrow;
use core::fmt;
use core::marker::PhantomData;
use core::task::{ready, Poll};

use alloc::vec::Vec;

use channels_packet::{Flags, Header, PayloadLength};
use channels_serdes::{PayloadBuffer, Serializer};

#[allow(unused_imports)]
use crate::common::{Pcb, Statistics};
use crate::error::SendError;
use crate::util::{Buf, IoSlice, PollExt};

/// The sending-half of the channel. This is the same as [`std::sync::mpsc::Sender`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
#[derive(Clone)]
pub struct Sender<T, W, S> {
	_marker: PhantomData<T>,
	writer: Writer<W>,
	serializer: S,
	pcb: Pcb,
}

impl<T> Sender<T, (), ()> {
	/// Create a new builder.
	pub const fn builder() -> Builder<T, (), ()> {
		Builder::new()
	}
}

#[cfg(feature = "bincode")]
impl<T, W> Sender<T, W, crate::serdes::Bincode> {
	/// Creates a new [`Sender`] from `writer`.
	pub fn new(writer: W) -> Self {
		Self::with_serializer(writer, Default::default())
	}
}

impl<T, W, S> Sender<T, W, S> {
	/// Create a new [`Sender`] from `writer` that uses `serializer`.
	pub fn with_serializer(writer: W, serializer: S) -> Self {
		Sender::builder()
			.writer(writer)
			.serializer(serializer)
			.build()
	}

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

impl<T, W, S> fmt::Debug for Sender<T, W, S>
where
	Writer<W>: fmt::Debug,
	S: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Sender")
			.field("writer", &self.writer)
			.field("serializer", &self.serializer)
			.finish_non_exhaustive()
	}
}

unsafe impl<T, W: Send, S: Send> Send for Sender<T, W, S> {}
unsafe impl<T, W: Sync, S: Sync> Sync for Sender<T, W, S> {}

/// A builder that when completed will return a [`Sender`].
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
	pub fn writer<W>(self, writer: W) -> Builder<T, W, S> {
		Builder {
			_marker: PhantomData,
			writer,
			serializer: self.serializer,
		}
	}
}

impl<T, W> Builder<T, W, ()> {
	/// Use this serializer.
	pub fn serializer<S>(self, serializer: S) -> Builder<T, W, S> {
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
			writer: Writer::new(self.writer),
			serializer: self.serializer,
			pcb: Pcb::new(),
		}
	}
}

#[derive(Debug, Clone)]
struct Writer<W> {
	inner: W,

	#[cfg(feature = "statistics")]
	statistics: Statistics,
}

impl<W> Writer<W> {
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

struct Packet {
	header: IoSlice<[u8; Header::SIZE]>,
	payload: IoSlice<Vec<u8>>,
}

impl Packet {
	pub fn has_remaining(&self) -> bool {
		self.header.has_remaining() || self.payload.has_remaining()
	}

	pub fn advance(&mut self, mut n: usize) {
		use core::cmp::min;

		if self.header.has_remaining() {
			let i = min(n, self.header.remaining());
			self.header.advance(i);
			n -= i;
		}

		if self.payload.has_remaining() {
			let i = min(n, self.payload.remaining());
			self.payload.advance(i);
			n -= i;
		}

		let _ = n;
	}
}

enum State {
	HasPacket { packet: Packet },
	NoPacket,
}

struct SendPayload<'a, W> {
	writer: &'a mut Writer<W>,
	pcb: &'a mut Pcb,
	data: PayloadBuffer,
	has_sent_at_least_one: bool,
	state: State,
}

impl<'a, W> SendPayload<'a, W> {
	fn new(
		pcb: &'a mut Pcb,
		writer: &'a mut Writer<W>,
		data: PayloadBuffer,
	) -> Self {
		Self {
			pcb,
			writer,
			data,
			has_sent_at_least_one: false,
			state: State::NoPacket,
		}
	}

	fn advance<F, E>(
		&mut self,
		mut write_packet: F,
	) -> Poll<Result<(), E>>
	where
		F: FnMut(&mut Writer<W>, &mut Packet) -> Poll<Result<(), E>>,
	{
		use Poll::*;

		loop {
			match self.state {
				State::NoPacket => {
					let (payload, payload_length) = match (
						self.data.pop_first(),
						self.has_sent_at_least_one,
					) {
						(Some(payload), _) => {
							debug_assert!(
								payload.len()
									<= PayloadLength::MAX.as_usize()
							);

							// SAFETY: `payload` comes from `self.data` which
							//         guarantees that its chunks have length
							//         `PayloadLength`.
							let len = payload.len() as u16;
							let len = unsafe {
								PayloadLength::new_unchecked(len)
							};

							(payload, len)
						},
						(None, false) => {
							(Vec::new(), PayloadLength::MIN)
						},
						(None, true) => return Ready(Ok(())),
					};

					let packet_length =
						payload_length.to_packet_length();

					let header = Header {
						length: packet_length,
						flags: Flags::zero()
							.set_if(Flags::MORE_DATA, |_| {
								self.data.chunk_count() > 0
							}),
						id: self.pcb.id_gen.next_id(),
					}
					.to_bytes();

					self.has_sent_at_least_one = true;

					self.state = State::HasPacket {
						packet: Packet {
							header: IoSlice::new(header),
							payload: IoSlice::new(payload),
						},
					};
				},
				State::HasPacket { ref mut packet } => {
					match ready!(write_packet(self.writer, packet)) {
						Err(e) => return Ready(Err(e)),
						Ok(_) => {
							if !packet.has_remaining() {
								self.state = State::NoPacket;
							}
						},
					}
				},
			}
		}
	}
}

#[cfg(all(feature = "tokio", feature = "futures"))]
core::compile_error!(
	"tokio and futures features cannot be active at once"
);

#[cfg(any(feature = "tokio", feature = "futures"))]
mod async_impl {
	use super::*;

	use core::future::Future;
	use core::pin::{pin, Pin};
	use core::task::{ready, Context, Poll};

	#[cfg(feature = "tokio")]
	mod imp {
		use super::*;

		use tokio::io;

		pub use io::{AsyncWrite, Error};

		pub fn poll_write_packet<W>(
			writer: &mut Writer<W>,
			packet: &mut Packet,
			cx: &mut Context,
		) -> Poll<io::Result<()>>
		where
			W: AsyncWrite + Unpin,
		{
			use io::ErrorKind as E;
			use Poll::*;

			while packet.has_remaining() {
				let iovecs = packet_to_iovecs(packet);

				match ready!(pin!(&mut writer.inner)
					.poll_write_vectored(cx, &iovecs))
				{
					Err(e) if e.kind() == E::Interrupted => continue,
					Err(e) if e.kind() == E::WouldBlock => {
						return Pending
					},
					Err(e) => return Ready(Err(e)),
					Ok(0) => {
						return Ready(Err(E::UnexpectedEof.into()))
					},
					Ok(n) => {
						packet.advance(n);
						writer.on_write(n as u64);
					},
				}
			}

			Ready(Ok(()))
		}
	}

	#[cfg(feature = "futures")]
	mod imp {
		use super::*;

		use std::io;

		pub use futures::AsyncWrite;
		pub use io::Error;

		pub fn poll_write_packet<W>(
			writer: &mut Writer<W>,
			packet: &mut Packet,
			cx: &mut Context,
		) -> Poll<io::Result<()>>
		where
			W: AsyncWrite + Unpin,
		{
			use io::ErrorKind as E;
			use Poll::*;

			while packet.has_remaining() {
				let iovecs = packet_to_iovecs(packet);

				match ready!(pin!(&mut writer.inner)
					.poll_write_vectored(cx, &iovecs))
				{
					Err(e) if e.kind() == E::Interrupted => continue,
					Err(e) if e.kind() == E::WouldBlock => {
						return Pending
					},
					Err(e) => return Ready(Err(e)),
					Ok(0) => {
						return Ready(Err(E::UnexpectedEof.into()))
					},
					Ok(n) => {
						packet.advance(n);
						writer.on_write(n as u64);
					},
				}
			}

			Ready(Ok(()))
		}
	}

	use self::imp::{poll_write_packet, AsyncWrite, Error};

	impl<W> Future for SendPayload<'_, W>
	where
		W: AsyncWrite + Unpin,
	{
		type Output = Result<(), Error>;

		fn poll(
			mut self: Pin<&mut Self>,
			cx: &mut Context<'_>,
		) -> Poll<Self::Output> {
			self.advance(|w, b| poll_write_packet(w, b, cx))
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
		) -> Result<(), SendError<S::Error, Error>>
		where
			D: Borrow<T>,
		{
			let payload = self
				.serializer
				.serialize(data.borrow())
				.map_err(SendError::Serde)?;

			SendPayload::new(&mut self.pcb, &mut self.writer, payload)
				.await
				.map_err(SendError::Io)
		}
	}
}

mod sync_impl {
	use super::*;

	mod imp {
		use super::*;

		use std::io;

		pub use io::{Error, Write};

		pub fn write_packet<W>(
			writer: &mut Writer<W>,
			packet: &mut Packet,
		) -> Poll<Result<(), Error>>
		where
			W: Write,
		{
			use io::ErrorKind as E;
			use Poll::*;

			while packet.has_remaining() {
				let iovecs = packet_to_iovecs(packet);

				match writer.inner.write_vectored(&iovecs) {
					Err(e) if e.kind() == E::Interrupted => continue,
					Err(e) if e.kind() == E::WouldBlock => {
						return Pending
					},
					Err(e) => return Ready(Err(e)),
					Ok(0) => return Ready(Err(E::WriteZero.into())),
					Ok(n) => {
						packet.advance(n);
						writer.on_write(n as u64);
					},
				}
			}

			Ready(Ok(()))
		}
	}

	use self::imp::{write_packet, Error, Write};

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
		) -> Result<(), SendError<S::Error, Error>>
		where
			D: Borrow<T>,
		{
			let payload = self
				.serializer
				.serialize(data.borrow())
				.map_err(SendError::Serde)?;

			SendPayload::new(&mut self.pcb, &mut self.writer, payload)
				.advance(write_packet)
				.unwrap()
				.map_err(SendError::Io)
		}
	}
}

#[inline]
fn packet_to_iovecs(packet: &mut Packet) -> [std::io::IoSlice; 2] {
	use std::io::IoSlice;

	[
		IoSlice::new(packet.header.unfilled()),
		IoSlice::new(packet.payload.unfilled()),
	]
}
