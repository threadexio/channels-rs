//! Module containing the implementation for [`Sender`].

use core::borrow::Borrow;
use core::fmt;
use core::marker::PhantomData;

#[allow(unused_imports)]
use crate::common::{Pcb, Statistics};
use crate::error::SendError;
use crate::io::{
	AsyncWrite, Buf, Contiguous, Cursor, IntoWriter, Write, Writer,
};
use crate::serdes::Serializer;

/// The sending-half of the channel.
pub struct Sender<T, W, S> {
	_marker: PhantomData<T>,
	writer: StatWriter<W>,
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
	pub fn new(writer: impl IntoWriter<W>) -> Self {
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
	pub fn with_serializer(
		writer: impl IntoWriter<W>,
		serializer: S,
	) -> Self {
		Sender::builder()
			.writer(writer)
			.serializer(serializer)
			.build()
	}
}

impl<T, W, S> Sender<T, W, S>
where
	W: Writer,
{
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
	pub fn get(&self) -> &W::Inner {
		self.writer.inner.get()
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
	pub fn get_mut(&mut self) -> &mut W::Inner {
		self.writer.inner.get_mut()
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

impl<T, W, S> Sender<T, W, S>
where
	W: AsyncWrite,
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
		let payload = self
			.serializer
			.serialize(data.borrow())
			.map_err(SendError::Serde)?;

		send::send_async(&mut self.pcb, &mut self.writer, payload)
			.await
			.map_err(SendError::Io)?;

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
	pub fn send_blocking<D>(
		&mut self,
		data: D,
	) -> Result<(), SendError<S::Error, W::Error>>
	where
		D: Borrow<T>,
	{
		let payload = self
			.serializer
			.serialize(data.borrow())
			.map_err(SendError::Serde)?;

		send::send(&mut self.pcb, &mut self.writer, payload)
			.map_err(SendError::Io)?;

		self.writer.flush().map_err(SendError::Io)?;

		Ok(())
	}
}

impl<T, W, S> fmt::Debug for Sender<T, W, S>
where
	StatWriter<W>: fmt::Debug,
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
	StatWriter<W>: Send,
	S: Send,
{
}

unsafe impl<T, W, S> Sync for Sender<T, W, S>
where
	StatWriter<W>: Sync,
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
	pub fn writer<W>(
		self,
		writer: impl IntoWriter<W>,
	) -> Builder<T, W, S> {
		Builder {
			_marker: PhantomData,
			writer: writer.into_writer(),
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
	pub(crate) statistics: Statistics,
}

impl<W> StatWriter<W> {
	pub fn new(writer: W) -> Self {
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

impl<W: Write> Write for StatWriter<W> {
	type Error = W::Error;

	fn write<B>(&mut self, mut buf: B) -> Result<(), Self::Error>
	where
		B: Contiguous,
	{
		let l0 = buf.remaining();
		self.inner.write(&mut buf)?;
		let l1 = buf.remaining();

		let dl = usize::abs_diff(l0, l1);
		self.on_write(dl as u64);
		Ok(())
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
		self.inner.flush()
	}
}

impl<W: AsyncWrite> AsyncWrite for StatWriter<W> {
	type Error = W::Error;

	async fn write<B>(
		&mut self,
		mut buf: B,
	) -> Result<(), Self::Error>
	where
		B: Contiguous,
	{
		let l0 = buf.remaining();
		self.inner.write(&mut buf).await?;
		let l1 = buf.remaining();

		let dl = usize::abs_diff(l0, l1);
		self.on_write(dl as u64);
		Ok(())
	}

	async fn flush(&mut self) -> Result<(), Self::Error> {
		self.inner.flush().await
	}
}

mod send {
	use super::*;

	use channels_packet::{
		Flags, Header, PacketLength, PayloadLength,
	};

	pub fn send<'a, W, B>(
		pcb: &'a mut Pcb,
		writer: &'a mut StatWriter<W>,
		payload: B,
	) -> Result<(), W::Error>
	where
		W: Write,
		B: Buf,
	{
		SendPayload::new(pcb, writer, payload).run()
	}

	pub async fn send_async<'a, W, B>(
		pcb: &'a mut Pcb,
		writer: &'a mut StatWriter<W>,
		payload: B,
	) -> Result<(), W::Error>
	where
		W: AsyncWrite,
		B: Buf,
	{
		SendPayload::new(pcb, writer, payload).run_async().await
	}

	struct SendPayload<'a, W, B> {
		pcb: &'a mut Pcb,
		writer: &'a mut StatWriter<W>,
		payload: B,
		has_sent_one_packet: bool,
	}

	impl<'a, W, B> SendPayload<'a, W, B> {
		pub fn new(
			pcb: &'a mut Pcb,
			writer: &'a mut StatWriter<W>,
			payload: B,
		) -> Self {
			Self { pcb, writer, payload, has_sent_one_packet: false }
		}
	}

	impl<'a, W, B> SendPayload<'a, W, B>
	where
		B: Buf,
	{
		/// Get the header for the next packet.
		///
		/// Returns [`Some`] with the header of the next packet that should be
		/// sent or [`None`] if no packet should be sent.
		fn next_header(&mut self) -> Option<Header> {
			match (self.payload.remaining(), self.has_sent_one_packet)
			{
				// If there is no more data and we have already sent
				// one packet, then exit.
				(0, true) => None,
				// If there is no more data and we have not sent any
				// packets, then send one packet with no payload.
				(0, false) => Some(Header {
					length: PacketLength::MIN,
					flags: Flags::zero(),
					id: self.pcb.id_gen.next_id(),
				}),
				// If the remaining data is more than what fits inside
				// one packet, return a header for a full packet.
				(rem, _) if rem > PayloadLength::MAX.as_usize() => {
					Some(Header {
						length: PayloadLength::MAX.to_packet_length(),
						flags: Flags::MORE_DATA,
						id: self.pcb.id_gen.next_id(),
					})
				},
				// If the remaining data is equal or less than what
				// fits inside one packet, return a header for exactly
				// that amount of data.
				#[allow(clippy::cast_possible_truncation)]
				(rem, _) => Some(Header {
					length: PayloadLength::new_saturating(rem as u16)
						.to_packet_length(),
					flags: Flags::zero(),
					id: self.pcb.id_gen.next_id(),
				}),
			}
		}
	}

	impl<'a, W, B> SendPayload<'a, W, B>
	where
		W: Write,
		B: Buf,
	{
		pub fn run(mut self) -> Result<(), W::Error> {
			while let Some(header) = self.next_header() {
				self.has_sent_one_packet = true;

				let payload_length =
					header.length.to_payload_length().as_usize();

				let header = Cursor::new(header.to_bytes());
				let payload =
					self.payload.by_ref().take(payload_length);
				let packet = channels_io::chain(header, payload);

				let mut packet = packet.copy_to_contiguous();
				self.writer.write(&mut packet)?;

				#[cfg(feature = "statistics")]
				self.writer.statistics.inc_packets();
			}

			#[cfg(feature = "statistics")]
			self.writer.statistics.inc_ops();

			Ok(())
		}
	}

	impl<'a, W, B> SendPayload<'a, W, B>
	where
		W: AsyncWrite,
		B: Buf,
	{
		pub async fn run_async(mut self) -> Result<(), W::Error> {
			while let Some(header) = self.next_header() {
				self.has_sent_one_packet = true;

				let payload_length =
					header.length.to_payload_length().as_usize();

				let header = Cursor::new(header.to_bytes());
				let payload =
					self.payload.by_ref().take(payload_length);
				let packet = channels_io::chain(header, payload);

				let mut packet = packet.copy_to_contiguous();
				self.writer.write(&mut packet).await?;

				#[cfg(feature = "statistics")]
				self.writer.statistics.inc_packets();
			}

			#[cfg(feature = "statistics")]
			self.writer.statistics.inc_ops();

			Ok(())
		}
	}
}
