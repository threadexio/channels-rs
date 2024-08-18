//! Module containing the implementation for [`Sender`].

use core::borrow::Borrow;
use core::fmt;
use core::marker::PhantomData;

use alloc::vec::Vec;

use channels_packet::frame::Frame;
use channels_packet::header::FrameNumSequence;
use channels_packet::payload::Payload;

use crate::error::SendError;
use crate::io::framed::{Encoder, FramedWrite, FramedWriteError};
use crate::io::sink::{AsyncSink, Sink};
use crate::io::{
	AsyncWrite, AsyncWriteExt, Container, IntoWrite, Write, WriteExt,
};
use crate::serdes::Serializer;
use crate::statistics::{StatIO, Statistics};

/// Configuration for [`Sender`].
#[derive(Clone)]
#[must_use = "`Config`s don't do anything on their own"]
pub struct Config {
	flags: u8,
}

impl Config {
	const FLUSH_ON_SEND: u8 = 1 << 0;

	#[inline]
	const fn get_flag(&self, flag: u8) -> bool {
		self.flags & flag != 0
	}

	#[inline]
	fn set_flag(&mut self, flag: u8, value: bool) {
		if value {
			self.flags |= flag;
		} else {
			self.flags &= !flag;
		}
	}
}

impl Default for Config {
	#[inline]
	fn default() -> Self {
		Self { flags: Self::FLUSH_ON_SEND }
	}
}

impl Config {
	/// TODO: docs
	#[inline]
	#[must_use]
	pub fn flush_on_send(&self) -> bool {
		self.get_flag(Self::FLUSH_ON_SEND)
	}

	/// TODO: docs
	#[inline]
	pub fn set_flush_on_send(&mut self, yes: bool) {
		self.set_flag(Self::FLUSH_ON_SEND, yes);
	}

	/// TODO: docs
	#[inline]
	pub fn with_flush_on_send(mut self, yes: bool) -> Self {
		self.set_flush_on_send(yes);
		self
	}
}

impl fmt::Debug for Config {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// TODO: fix config debug impl
		f.debug_struct("Config").finish()
	}
}

enum FrameEncodeError {
	TooLarge,
}

impl<Ser, Io> From<FramedWriteError<FrameEncodeError, Io>>
	for SendError<Ser, Io>
{
	fn from(value: FramedWriteError<FrameEncodeError, Io>) -> Self {
		use FrameEncodeError as C;
		use FramedWriteError as A;
		use SendError as B;

		match value {
			A::Encode(C::TooLarge) => B::TooLarge,
			A::Io(e) => B::Io(e),
		}
	}
}

struct FrameEncoder {
	config: Config,
	seq: FrameNumSequence,
}

impl Encoder for FrameEncoder {
	type Item = Vec<u8>;
	type Error = FrameEncodeError;

	fn encode(
		&mut self,
		item: Self::Item,
		buf: &mut Vec<u8>,
	) -> Result<(), Self::Error> {
		let payload = Payload::new(item)
			.map_err(|_| FrameEncodeError::TooLarge)?;

		let frame = Frame { payload, frame_num: self.seq.advance() };
		let header = frame.header().to_bytes();
		let payload = frame.payload;

		let frame_len = usize::checked_add(
			header.as_ref().len(),
			payload.as_ref().len(),
		)
		.ok_or(FrameEncodeError::TooLarge)?;

		buf.reserve(frame_len);
		buf.extend_from_slice(header.as_ref());
		buf.extend_from_slice(payload.as_ref());
		Ok(())
	}
}

/// The sending-half of the channel.
pub struct Sender<T, W, S> {
	_marker: PhantomData<fn() -> T>,
	serializer: S,
	framed: FramedWrite<StatIO<W>, FrameEncoder>,
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
	#[inline]
	pub fn builder() -> Builder<T, (), ()> {
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
	#[inline]
	pub fn new(writer: impl IntoWrite<W>) -> Self {
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
	#[inline]
	pub fn with_serializer(
		writer: impl IntoWrite<W>,
		serializer: S,
	) -> Self {
		Sender::builder()
			.writer(writer)
			.serializer(serializer)
			.build()
	}

	/// Get the config that was given to this [`Sender`].
	///
	/// # Example
	///
	/// ```no_run
	/// use channels::sender::{Config, Sender};
	/// use channels::serdes::Bincode;
	///
	/// let writer = std::io::sink();
	///
	/// let config = Config::default()
	///                 .with_flush_on_send(false);
	///
	/// let rx = Sender::<i32, _, _>::builder()
	///             .writer(writer)
	///             .serializer(Bincode::new())
	///             .config(config)
	///             .build();
	///
	/// println!("{:#?}", rx.config());
	///
	/// ```
	#[inline]
	pub fn config(&self) -> &Config {
		todo!()
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
	#[inline]
	#[cfg(feature = "statistics")]
	pub fn statistics(&self) -> &Statistics {
		&self.framed.writer().statistics
	}
}

impl<T, W, S> Sender<T, W, S>
where
	W: Container,
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
	#[inline]
	pub fn get(&self) -> &W::Inner {
		self.framed.writer().inner.get_ref()
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
	#[inline]
	pub fn get_mut(&mut self) -> &mut W::Inner {
		self.framed.writer_mut().inner.get_mut()
	}

	/// Destruct the sender and get back the underlying writer.
	#[inline]
	pub fn into_writer(self) -> W::Inner {
		self.framed.into_writer().inner.into_inner()
	}
}

impl<T, W, S> Sender<T, W, S>
where
	S: Serializer<T>,
{
	#[inline]
	fn serialize_t<Io>(
		&mut self,
		t: &T,
	) -> Result<Vec<u8>, SendError<S::Error, Io>> {
		self.serializer.serialize(t).map_err(SendError::Serde)
	}
}

impl<T, W, S> Sender<T, W, S>
where
	W: AsyncWrite + Unpin,
	S: Serializer<T>,
{
	/// Attempts to send `data` through the channel.
	///
	/// # Cancel Safety
	///
	/// This method is **NOT** cancel safe. Dropping this future means data might
	/// get lost.  If the method is used in some `select!`-like macro and some other
	/// branch completes first, then only a part of the data may have been sent
	/// over the channel.
	///
	/// # Example
	///
	/// ```no_run
	/// use tokio::net::TcpStream;
	///
	/// # #[tokio::main]
	/// # async fn main() {
	/// let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
	/// let mut tx = channels::Sender::<i32, _, _>::new(stream);
	///
	/// let data = 42;
	///
	/// tx.send(&data).await.unwrap();
	/// // or by taking ownership
	/// tx.send(data).await.unwrap();
	/// # }
	/// ```
	#[inline]
	pub async fn send<D>(
		&mut self,
		data: D,
	) -> Result<(), SendError<S::Error, W::Error>>
	where
		D: Borrow<T>,
	{
		self._send(data.borrow()).await
	}

	#[inline(never)]
	async fn _send(
		&mut self,
		data: &T,
	) -> Result<(), SendError<S::Error, W::Error>> {
		let payload = self.serialize_t(data)?;
		self.framed.send(payload).await.map_err(SendError::from)?;
		self.framed.writer_mut().statistics.inc_total_items();

		if self.framed.encoder().config.flush_on_send() {
			self.framed
				.writer_mut()
				.flush()
				.await
				.map_err(SendError::Io)?;
		}

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
	/// # Non-blocking IO
	///
	/// Non-blocking writers (those who return `WouldBlock`) are _not_ supported
	/// and will _not_ work. If you want non-blocking operation prefer the asynchronous
	/// version of this function, [`send()`].
	///
	/// # Example
	///
	/// ```no_run
	/// use std::net::TcpStream;
	///
	/// let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
	/// let mut tx = channels::Sender::<i32, _, _>::new(stream);
	///
	/// let data = 42;
	///
	/// tx.send_blocking(&data).unwrap();
	/// // or by taking ownership
	/// tx.send_blocking(data).unwrap();
	/// ```
	///
	/// [`send()`]: fn@Sender::send
	#[inline]
	pub fn send_blocking<D>(
		&mut self,
		data: D,
	) -> Result<(), SendError<S::Error, W::Error>>
	where
		D: Borrow<T>,
	{
		self._send_blocking(data.borrow())
	}

	#[inline(never)]
	fn _send_blocking(
		&mut self,
		data: &T,
	) -> Result<(), SendError<S::Error, W::Error>> {
		let payload = self.serialize_t(data)?;
		self.framed.send(payload).map_err(SendError::from)?;
		self.framed.writer_mut().statistics.inc_total_items();

		if self.framed.encoder().config.flush_on_send() {
			self.framed
				.writer_mut()
				.flush()
				.map_err(SendError::Io)?;
		}

		Ok(())
	}
}

impl<T, W, S> fmt::Debug for Sender<T, W, S>
where
	W: fmt::Debug,
	S: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Sender")
			.field("writer", self.framed.writer())
			.field("serializer", &self.serializer)
			.field("config", &self.config())
			.finish_non_exhaustive()
	}
}

/// A builder for [`Sender`].
#[derive(Clone)]
#[must_use = "builders don't do anything unless you `.build()` them"]
pub struct Builder<T, W, S> {
	_marker: PhantomData<fn() -> T>,
	writer: W,
	serializer: S,
	config: Option<Config>,
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
	#[inline]
	pub fn new() -> Self {
		Builder {
			_marker: PhantomData,
			serializer: (),
			writer: (),
			config: None,
		}
	}
}

impl<T> Default for Builder<T, (), ()> {
	#[inline]
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
	#[inline]
	pub fn writer<W>(
		self,
		writer: impl IntoWrite<W>,
	) -> Builder<T, W, S> {
		Builder {
			_marker: PhantomData,
			writer: writer.into_write(),
			serializer: self.serializer,
			config: self.config,
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
	#[inline]
	pub fn serializer<S>(self, serializer: S) -> Builder<T, W, S> {
		Builder {
			_marker: PhantomData,
			writer: self.writer,
			serializer,
			config: self.config,
		}
	}
}

impl<T, W, S> Builder<T, W, S> {
	/// Set the [`Config`] for this sender.
	///
	/// # Example
	///
	/// ```no_run
	/// use channels::sender::{Config, Sender};
	///
	/// let config = Config::default()
	///                 .with_flush_on_send(false);
	///
	/// let tx = Sender::<i32, _, _>::builder()
	///             .config(config);
	/// ```
	#[inline]
	pub fn config(mut self, config: Config) -> Self {
		self.config = Some(config);
		self
	}

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
	#[inline]
	pub fn build(self) -> Sender<T, W, S> {
		let Self { _marker, config, serializer, writer } = self;

		Sender {
			_marker: PhantomData,
			serializer,
			framed: FramedWrite::new(
				StatIO::new(writer),
				FrameEncoder {
					config: config.unwrap_or_default(),
					seq: FrameNumSequence::new(),
				},
			),
		}
	}
}

impl<T, W, S> fmt::Debug for Builder<T, W, S>
where
	W: fmt::Debug,
	S: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Builder")
			.field("writer", &self.writer)
			.field("serializer", &self.serializer)
			.field("config", &self.config)
			.finish()
	}
}
