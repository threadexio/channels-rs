//! Module containing the implementation for [`Sender`].

use core::borrow::Borrow;
use core::fmt;
use core::marker::PhantomData;
use core::task::{ready, Poll};

use alloc::vec::Vec;

use channels_packet::{Flags, Header, PayloadLength};
use channels_serdes::Serializer;

#[allow(unused_imports)]
use crate::common::{Pcb, Statistics};
use crate::error::SendError;
use crate::util::{Buf, IoSlice, PollExt};

/// The sending-half of the channel.
pub struct Sender<T, W, S> {
	_marker: PhantomData<T>,
	writer: Writer<W>,
	serializer: S,
	pcb: Pcb,
}

impl<T> Sender<T, (), ()> {
	/// Create a new builder.
	///
	/// # Example
	///
	/// ```no_run
	/// let writer = std::io::sink();
	/// let serializer = channels::serdes::Bincode::new();
	///
	/// let tx = channels::Sender::<i32, _, _>::builder()
	///            .writer(writer)
	///            .serializer(serializer)
	///            .build();
	/// ```
	#[must_use]
	pub const fn builder() -> Builder<T, (), ()> {
		Builder::new()
	}
}

#[cfg(feature = "bincode")]
impl<T, W> Sender<T, W, crate::serdes::Bincode> {
	/// Creates a new [`Sender`] from `writer`.
	///
	/// This constructor is a shorthand for calling [`Sender::builder()`] with
	/// `writer` and the default serializer, which is [`Bincode`].
	///
	/// # Example
	///
	/// Synchronously:
	///
	/// ```no_run
	/// let writer = std::io::sink();
	/// let tx = channels::Sender::<i32, _, _>::new(writer);
	/// ```
	///
	/// Asynchronously:
	///
	/// ```no_run
	/// let writer = tokio::io::sink();
	/// let tx = channels::Sender::<i32, _, _>::new(writer);
	/// ```
	///
	/// [`Bincode`]: crate::serdes::Bincode
	pub fn new(writer: W) -> Self {
		Self::with_serializer(writer, crate::serdes::Bincode::new())
	}
}

impl<T, W, S> Sender<T, W, S> {
	/// Create a new [`Sender`] from `writer` that uses `serializer`.
	///
	/// This constructor is a shorthand for calling [`Sender::builder()`] with
	/// `writer` and `serializer`.
	///
	/// # Example
	///
	/// Synchronously:
	///
	/// ```no_run
	/// let serializer = channels::serdes::Bincode::new();
	/// let writer = std::io::sink();
	///
	/// let tx = channels::Sender::<i32, _, _>::with_serializer(
	///     writer,
	///     serializer
	/// );
	/// ```
	///
	/// Asynchronously:
	///
	/// ```no_run
	/// let serializer = channels::serdes::Bincode::new();
	/// let writer = tokio::io::sink();
	///
	/// let tx = channels::Sender::<i32, _, _>::with_serializer(
	///     writer,
	///     serializer
	/// );
	/// ```
	pub fn with_serializer(writer: W, serializer: S) -> Self {
		Sender::builder()
			.writer(writer)
			.serializer(serializer)
			.build()
	}

	/// Get a reference to the underlying writer.
	///
	/// # Example
	///
	/// ```
	/// use std::io;
	///
	/// struct MyWriter {
	///     count: usize
	/// }
	///
	/// impl io::Write for MyWriter {
	///     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
	///         self.count += 1;
	///         Ok(buf.len())
	///     }
	///
	///     fn flush(&mut self) -> io::Result<()> {
	///         Ok(())
	///     }
	/// }
	///
	/// let tx = channels::Sender::<i32, _, _>::new(MyWriter { count: 42 });
	///
	/// let w: &MyWriter = tx.get();
	/// assert_eq!(w.count, 42);
	/// ```
	pub fn get(&self) -> &W {
		&self.writer.inner
	}

	/// Get a mutable reference to the underlying writer. Directly writing to
	/// the stream is not advised.
	///
	/// # Example
	///
	/// ```
	/// use std::io;
	///
	/// struct MyWriter {
	///     count: usize
	/// }
	///
	/// impl io::Write for MyWriter {
	///     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
	///         self.count += 1;
	///         Ok(buf.len())
	///     }
	///
	///     fn flush(&mut self) -> io::Result<()> {
	///         Ok(())
	///     }
	/// }
	///
	/// let mut tx = channels::Sender::<i32, _, _>::new(MyWriter { count: 42 });
	///
	/// let w: &mut MyWriter = tx.get_mut();
	/// w.count += 10;
	/// assert_eq!(w.count, 52);
	/// ```
	pub fn get_mut(&mut self) -> &mut W {
		&mut self.writer.inner
	}

	/// Get statistics on this sender.
	///
	/// # Example
	///
	/// ```
	/// let writer = std::io::sink();
	/// let tx = channels::Sender::<i32, _, _>::new(writer);
	///
	/// let stats = tx.statistics();
	/// assert_eq!(stats.total_bytes(), 0);
	/// assert_eq!(stats.packets(), 0);
	/// assert_eq!(stats.ops(), 0);
	/// ```
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

unsafe impl<T, W, S> Send for Sender<T, W, S>
where
	Writer<W>: Send,
	S: Send,
{
}

unsafe impl<T, W, S> Sync for Sender<T, W, S>
where
	Writer<W>: Sync,
	S: Sync,
{
}

/// A builder for [`Sender`].
pub struct Builder<T, W, S> {
	_marker: PhantomData<T>,
	writer: W,
	serializer: S,
}

impl<T, R, D> fmt::Debug for Builder<T, R, D>
where
	R: fmt::Debug,
	D: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Builder")
			.field("writer", &self.writer)
			.field("serializer", &self.serializer)
			.finish()
	}
}

impl<T> Builder<T, (), ()> {
	/// Create a new [`Builder`] with the default options.
	///
	/// # Example
	///
	/// ```no_run
	/// let writer = std::io::sink();
	/// let serializer = channels::serdes::Bincode::new();
	///
	/// let tx = channels::sender::Builder::<i32, _, _>::new()
	///            .writer(writer)
	///            .serializer(serializer)
	///            .build();
	/// ```
	#[must_use]
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
	/// Sets the writer of the [`Sender`].
	///
	/// This function accepts both synchronous and asynchronous writers.
	///
	/// # Example
	///
	/// Synchronously:
	///
	/// ```no_run
	/// let builder = channels::Sender::<i32, _, _>::builder()
	///                 .writer(std::io::sink());
	/// ```
	///
	/// Asynchronously:
	///
	/// ```no_run
	/// let builder = channels::Sender::<i32, _, _>::builder()
	///                 .writer(tokio::io::sink());
	/// ```
	pub fn writer<W>(self, writer: W) -> Builder<T, W, S> {
		Builder {
			_marker: PhantomData,
			writer,
			serializer: self.serializer,
		}
	}
}

impl<T, W> Builder<T, W, ()> {
	/// Sets the serializer of the [`Sender`].
	///
	/// # Example
	///
	/// ```no_run
	/// let serializer = channels::serdes::Bincode::new();
	///
	/// let builder = channels::Sender::<i32, _, _>::builder()
	///                 .serializer(serializer);
	/// ```
	pub fn serializer<S>(self, serializer: S) -> Builder<T, W, S> {
		Builder {
			_marker: PhantomData,
			writer: self.writer,
			serializer,
		}
	}
}

impl<T, W, S> Builder<T, W, S> {
	/// Build a [`Sender`].
	///
	/// # Example
	///
	/// ```no_run
	/// let tx: channels::Sender<i32, _, _> = channels::Sender::builder()
	///            .writer(std::io::sink())
	///            .serializer(channels::serdes::Bincode::new())
	///            .build();
	/// ```
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

struct Packet<'a> {
	header: &'a mut dyn Buf,
	payload: &'a mut dyn Buf,
}

impl Packet<'_> {
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

	pub fn as_iovecs(&mut self) -> [std::io::IoSlice; 2] {
		use std::io::IoSlice;

		[
			IoSlice::new(self.header.unfilled()),
			IoSlice::new(self.payload.unfilled()),
		]
	}
}

enum State {
	SendPacket {
		header: IoSlice<[u8; Header::SIZE]>,
		payload_left: usize,
	},
	NeedsPacket,
}

struct SendPayload<'a, W> {
	writer: &'a mut Writer<W>,
	pcb: &'a mut Pcb,
	data: IoSlice<Vec<u8>>,
	has_sent_at_least_one: bool,
	state: State,
}

impl<'a, W> SendPayload<'a, W> {
	fn new(
		pcb: &'a mut Pcb,
		writer: &'a mut Writer<W>,
		data: Vec<u8>,
	) -> Self {
		Self {
			pcb,
			writer,
			data: IoSlice::new(data),
			has_sent_at_least_one: false,
			state: State::NeedsPacket,
		}
	}

	fn advance<F, E>(
		&mut self,
		mut write_packet: F,
	) -> Poll<Result<(), E>>
	where
		F: FnMut(&mut Writer<W>, &mut Packet) -> Poll<Result<(), E>>,
	{
		use Poll::Ready;

		use core::cmp::min;

		loop {
			match self.state {
				State::NeedsPacket => {
					let (payload_length, has_more) = match (
						self.data.has_remaining(),
						self.has_sent_at_least_one,
					) {
						(false, false) => (PayloadLength::MIN, false),
						(false, true) => {
							#[cfg(feature = "statistics")]
							self.writer.statistics.inc_ops();

							return Ready(Ok(()));
						},
						(true, _) => {
							let len = min(
								PayloadLength::MAX.as_usize(),
								self.data.remaining(),
							);

							let has_more =
								self.data.remaining() > len;

							// SAFETY: `len` is capped at `PayloadLength::MAX`,
							//         so it definitely fits inside a u16 and is
							//         definitely inside the required range.
							//         See: `PayloadLength::new`.
							#[allow(clippy::cast_possible_truncation)]
							let len = PayloadLength::new_saturating(
								len as u16,
							);

							(len, has_more)
						},
					};

					let header = Header {
						length: payload_length.to_packet_length(),
						flags: Flags::zero()
							.set_if(Flags::MORE_DATA, |_| has_more),
						id: self.pcb.id_gen.next_id(),
					};

					self.has_sent_at_least_one = true;

					self.state = State::SendPacket {
						header: IoSlice::new(header.to_bytes()),
						payload_left: payload_length.as_usize(),
					};
				},
				State::SendPacket {
					ref mut header,
					ref mut payload_left,
				} => {
					struct PayloadBuf<'a> {
						inner: &'a mut IoSlice<Vec<u8>>,
						left: &'a mut usize,
					}

					impl Buf for PayloadBuf<'_> {
						fn remaining(&self) -> usize {
							min(self.inner.remaining(), *self.left)
						}

						fn unfilled(&self) -> &[u8] {
							match self.inner.unfilled() {
								s if s.len() > *self.left => {
									&s[..*self.left]
								},
								s => s,
							}
						}

						fn advance(&mut self, n: usize) {
							let n = min(n, *self.left);
							*self.left -= n;
							self.inner.advance(n);
						}
					}

					let mut payload = PayloadBuf {
						inner: &mut self.data,
						left: payload_left,
					};

					let mut packet =
						Packet { header, payload: &mut payload };

					ready!(write_packet(self.writer, &mut packet))?;

					if !packet.has_remaining() {
						self.state = State::NeedsPacket;

						#[cfg(feature = "statistics")]
						self.writer.statistics.inc_packets();
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
	use super::{
		Borrow, Packet, SendError, SendPayload, Sender, Serializer,
		Writer,
	};

	use core::future::Future;
	use core::pin::{pin, Pin};
	use core::task::{ready, Context, Poll};

	#[cfg(feature = "tokio")]
	mod imp {
		use super::{pin, ready, Context, Packet, Poll, Writer};

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
			use Poll::{Pending, Ready};

			while packet.has_remaining() {
				let iovecs = packet.as_iovecs();

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
	use super::{
		Borrow, Packet, Poll, PollExt, SendError, SendPayload,
		Sender, Serializer, Writer,
	};

	mod imp {
		use super::{Packet, Poll, Writer};

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
			use Poll::{Pending, Ready};

			while packet.has_remaining() {
				let iovecs = packet.as_iovecs();

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
