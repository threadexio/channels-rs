//! Module containing the implementation for [`Receiver`].

use core::fmt;
use core::marker::PhantomData;
use core::num::NonZeroUsize;

use channels_packet::header::{FrameNumSequence, HeaderError};
use channels_packet::Header;

use crate::error::RecvError;
use crate::io::framed::{Decoder, FramedRead, FramedReadError};
use crate::io::source::{AsyncSource, Source};
use crate::io::{
	AsyncRead, AsyncReadExt, Container, IntoRead, Read, ReadExt,
};
use crate::serdes::Deserializer;
use crate::statistics::{StatIO, Statistics};

/// Configuration for [`Receiver`].
#[derive(Clone)]
#[must_use = "`Config`s don't do anything on their own"]
pub struct Config {
	pub(crate) max_size: Option<NonZeroUsize>,
	pub(crate) flags: u8,
}

impl Config {
	const VERIFY_ORDER: u8 = 1 << 0;

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
		Self { max_size: None, flags: Self::VERIFY_ORDER }
	}
}

impl Config {
	/// Get the max payload size the [`Receiver`] will accept.
	#[inline]
	#[must_use]
	pub fn max_size(&self) -> usize {
		self.max_size.map_or(0, NonZeroUsize::get)
	}

	/// Set the max payload size the [`Receiver`] will accept.
	#[allow(clippy::missing_panics_doc)]
	#[inline]
	pub fn set_max_size(&mut self, max_size: usize) -> &mut Self {
		self.max_size = match max_size {
			0 => None,
			x => Some(
				NonZeroUsize::new(x)
					.expect("max_size should never be 0"),
			),
		};
		self
	}

	/// Set the max payload size the [`Receiver`] will accept.
	#[inline]
	pub fn with_max_size(mut self, max_size: usize) -> Self {
		self.set_max_size(max_size);
		self
	}

	/// Check whether the [`Receiver`] will verify the order of received frames.
	#[inline]
	#[must_use]
	pub fn verify_order(&self) -> bool {
		self.get_flag(Self::VERIFY_ORDER)
	}

	/// Set whether the [`Receiver`] will verify the order of received frames.
	pub fn set_verify_order(&mut self, yes: bool) -> &mut Self {
		self.set_flag(Self::VERIFY_ORDER, yes);
		self
	}

	/// Set whether the [`Receiver`] will verify the order of received frames.
	pub fn with_verify_order(mut self, yes: bool) -> Self {
		self.set_verify_order(yes);
		self
	}
}

impl fmt::Debug for Config {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Config")
			.field("max_size", &self.max_size())
			.finish()
	}
}

enum FrameDecodeError {
	InvalidChecksum,
	OutOfOrder,
	TooLarge,
	VersionMismatch,
}

impl From<HeaderError> for FrameDecodeError {
	fn from(err: HeaderError) -> Self {
		use FrameDecodeError as B;
		use HeaderError as A;

		match err {
			A::InvalidChecksum => B::InvalidChecksum,
			A::VersionMismatch => B::VersionMismatch,
		}
	}
}

impl<Des, Io> From<FramedReadError<FrameDecodeError, Io>>
	for RecvError<Des, Io>
{
	fn from(value: FramedReadError<FrameDecodeError, Io>) -> Self {
		use FrameDecodeError as C;
		use FramedReadError as A;
		use RecvError as B;

		match value {
			A::Decode(C::OutOfOrder) => B::OutOfOrder,
			A::Decode(C::InvalidChecksum) => B::InvalidChecksum,
			A::Decode(C::TooLarge) => B::TooLarge,
			A::Decode(C::VersionMismatch) => B::VersionMismatch,
			A::Io(e) => B::Io(e),
		}
	}
}

struct FrameDecoder {
	config: Config,
	seq: FrameNumSequence,
}

impl Decoder for FrameDecoder {
	type Output = Vec<u8>;
	type Error = FrameDecodeError;

	fn decode(
		&mut self,
		buf: &mut Vec<u8>,
	) -> Result<Option<Self::Output>, Self::Error> {
		let Some(hdr) = Header::try_parse(buf.as_slice())? else {
			buf.reserve(Header::MAX_SIZE - buf.len());
			return Ok(None);
		};

		let hdr_len = hdr.length();

		let payload_len: usize = hdr
			.data_len
			.get()
			.try_into()
			.map_err(|_| FrameDecodeError::TooLarge)?;

		let frame_len = hdr_len
			.checked_add(payload_len)
			.ok_or(FrameDecodeError::TooLarge)?;

		if let Some(max_size) = self.config.max_size {
			if payload_len > max_size.get() {
				return Err(FrameDecodeError::TooLarge);
			}
		}

		if self.config.verify_order()
			&& hdr.frame_num != self.seq.peek()
		{
			return Err(FrameDecodeError::OutOfOrder);
		}

		if buf.len() < frame_len {
			buf.reserve(frame_len - buf.len());
			return Ok(None);
		}

		let payload = buf[hdr_len..frame_len].to_vec();

		let _ = self.seq.advance();
		buf.drain(..frame_len);
		Ok(Some(payload))
	}
}

/// The receiving-half of the channel.
pub struct Receiver<T, R, D> {
	_marker: PhantomData<fn() -> T>,
	deserializer: D,
	framed: FramedRead<StatIO<R>, FrameDecoder>,
}

impl<T> Receiver<T, (), ()> {
	/// Create a new builder.
	///
	/// # Example
	///
	/// ```no_run
	/// let reader = std::io::empty();
	/// let deserializer = channels::serdes::Bincode::new();
	///
	/// let rx = channels::Receiver::<i32, _, _>::builder()
	///            .reader(reader)
	///            .deserializer(deserializer)
	///            .build();
	/// ```
	#[inline]
	pub fn builder() -> Builder<T, (), ()> {
		Builder::new()
	}
}

#[cfg(feature = "bincode")]
impl<T, R> Receiver<T, R, crate::serdes::Bincode> {
	/// Creates a new [`Receiver`] from `reader`.
	///
	/// This constructor is a shorthand for calling [`Receiver::builder()`] with
	/// `reader` and the default deserializer, which is [`Bincode`].
	///
	/// # Example
	///
	/// Synchronously:
	///
	/// ```no_run
	/// let reader = std::io::empty();
	/// let rx = channels::Receiver::<i32, _, _>::new(reader);
	/// ```
	///
	/// Asynchronously:
	///
	/// ```no_run
	/// let reader = tokio::io::empty();
	/// let rx = channels::Receiver::<i32, _, _>::new(reader);
	/// ```
	///
	/// [`Bincode`]: crate::serdes::Bincode
	#[inline]
	pub fn new(reader: impl IntoRead<R>) -> Self {
		Self::with_deserializer(reader, crate::serdes::Bincode::new())
	}
}

impl<T, R, D> Receiver<T, R, D> {
	/// Create a new [`Receiver`] from `reader` that uses `deserializer`.
	///
	/// This constructor is a shorthand for calling [`Receiver::builder()`] with
	/// `reader` and `deserializer`.
	///
	/// # Example
	///
	/// Synchronously:
	///
	/// ```no_run
	/// let deserializer = channels::serdes::Bincode::new();
	/// let reader = std::io::empty();
	///
	/// let rx = channels::Receiver::<i32, _, _>::with_deserializer(
	///     reader,
	///     deserializer
	/// );
	/// ```
	///
	/// Asynchronously:
	///
	/// ```no_run
	/// let deserializer = channels::serdes::Bincode::new();
	/// let reader = tokio::io::empty();
	///
	/// let rx = channels::Receiver::<i32, _, _>::with_deserializer(
	///     reader,
	///     deserializer
	/// );
	/// ```
	#[inline]
	pub fn with_deserializer(
		reader: impl IntoRead<R>,
		deserializer: D,
	) -> Self {
		Receiver::builder()
			.reader(reader)
			.deserializer(deserializer)
			.build()
	}

	/// Get the config that was given to this [`Receiver`].
	///
	/// # Example
	///
	/// ```no_run
	/// use channels::receiver::{Config, Receiver};
	/// use channels::serdes::Bincode;
	///
	/// let reader = std::io::empty();
	///
	/// let config = Config::default()
	///                 .with_size_estimate(42);
	///
	/// let rx = Receiver::<i32, _, _>::builder()
	///             .reader(reader)
	///             .deserializer(Bincode::new())
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

	/// Get an iterator over incoming messages.
	///
	/// # Example
	///
	/// Synchronously:
	///
	/// ```no_run
	/// let reader = std::io::empty();
	/// let mut rx = channels::Receiver::<i32, _, _>::new(reader);
	///
	/// for message in rx.incoming() {
	///     match message {
	///         Ok(message) => println!("received: {message}"),
	///         Err(err) => eprintln!("failed to receive message: {err}"),
	///     }
	/// }
	/// ```
	///
	/// Asynchronously:
	///
	/// ```no_run
	/// # #[tokio::main]
	/// # async fn main() {
	/// let reader = tokio::io::empty();
	/// let mut rx = channels::Receiver::<i32, _, _>::new(reader);
	///
	/// let mut incoming = rx.incoming();
	///
	/// loop {
	///     tokio::select! {
	///         message = incoming.next_async() => {
	///             match message {
	///                 Ok(message) => println!("received: {message}"),
	///                 Err(err) => eprintln!("failed to receive message: {err}"),
	///             }
	///         }
	///         // ...
	///     }
	/// }
	/// # }
	/// ```
	#[inline]
	pub fn incoming(&mut self) -> Incoming<'_, T, R, D> {
		Incoming { receiver: self }
	}

	/// Get statistics on this receiver.
	///
	/// # Example
	///
	/// ```
	/// let reader = std::io::empty();
	/// let rx = channels::Receiver::<i32, _, _>::new(reader);
	///
	/// let stats = rx.statistics();
	/// assert_eq!(stats.total_bytes(), 0);
	/// assert_eq!(stats.packets(), 0);
	/// assert_eq!(stats.ops(), 0);
	/// ```
	#[inline]
	#[cfg(feature = "statistics")]
	pub fn statistics(&self) -> &Statistics {
		&self.framed.reader().statistics
	}
}

impl<T, R, D> Receiver<T, R, D>
where
	R: Container,
{
	/// Get a reference to the underlying reader.
	///
	/// # Example
	///
	/// ```
	/// use std::io;
	///
	/// struct MyReader {
	///     count: usize
	/// }
	///
	/// impl io::Read for MyReader {
	///     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
	///         self.count += 1;
	///         Ok(buf.len())
	///     }
	/// }
	///
	/// let rx = channels::Receiver::<i32, _, _>::new(MyReader { count: 42 });
	///
	/// let r: &MyReader = rx.get();
	/// assert_eq!(r.count, 42);
	/// ```
	#[inline]
	pub fn get(&self) -> &R::Inner {
		self.framed.reader().inner.get_ref()
	}

	/// Get a mutable reference to the underlying reader. Directly reading from
	/// the stream is not advised.
	///
	/// # Example
	///
	/// ```
	/// use std::io;
	///
	/// struct MyReader {
	///     count: usize
	/// }
	///
	/// impl io::Read for MyReader {
	///     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
	///         self.count += 1;
	///         Ok(buf.len())
	///     }
	/// }
	///
	/// let mut rx = channels::Receiver::<i32, _, _>::new(MyReader { count: 42 });
	///
	/// let r: &mut MyReader = rx.get_mut();
	/// r.count += 10;
	/// assert_eq!(r.count, 52);
	/// ```
	#[inline]
	pub fn get_mut(&mut self) -> &mut R::Inner {
		self.framed.reader_mut().inner.get_mut()
	}

	/// Destruct the receiver and get back the underlying reader.
	#[inline]
	pub fn into_reader(self) -> R::Inner {
		self.framed.into_reader().inner.into_inner()
	}
}

impl<T, R, D> Receiver<T, R, D>
where
	R: AsyncRead + Unpin,
	D: Deserializer<T>,
{
	/// Attempts to receive a type `T` from the channel.
	///
	/// # Cancel Safety
	///
	/// This method is cancel safe. If the method is used as the event in some
	/// `select!`-like macro and some other branch completes first, then it is
	/// guaranteed that no data will be dropped.
	///
	/// # Example
	///
	/// ```no_run
	/// use tokio::net::TcpStream;
	///
	/// # #[tokio::main]
	/// # async fn main() {
	/// let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
	/// let mut rx = channels::Receiver::<i32, _, _>::new(stream);
	///
	/// let received: i32 = rx.recv().await.unwrap();
	/// println!("{received}");
	/// # }
	/// ```
	///
	/// [`recv()`]: fn@Self::recv
	pub async fn recv(
		&mut self,
	) -> Result<T, RecvError<D::Error, R::Error>> {
		let mut payload =
			self.framed.next().await.map_err(RecvError::from)?;
		self.framed.reader_mut().statistics.inc_total_items();

		self.deserializer
			.deserialize(&mut payload)
			.map_err(RecvError::Serde)
	}
}

impl<T, R, D> Receiver<T, R, D>
where
	R: Read,
	D: Deserializer<T>,
{
	/// Attempts to receive a type `T` from the channel.
	///
	/// # Non-blocking IO
	///
	/// Non-blocking readers (those who return `WouldBlock`) are _not_ supported
	/// and will _not_ work. If you want non-blocking operation prefer the asynchronous
	/// version of this function, [`recv()`].
	///
	/// # Example
	///
	/// ```no_run
	/// use std::net::TcpStream;
	///
	/// let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
	/// let mut rx = channels::Receiver::<i32, _, _>::new(stream);
	///
	/// let received: i32 = rx.recv_blocking().unwrap();
	/// println!("{received}");
	/// ```
	///
	/// [`recv()`]: fn@Receiver::recv
	#[inline]
	pub fn recv_blocking(
		&mut self,
	) -> Result<T, RecvError<D::Error, R::Error>> {
		let mut payload =
			self.framed.next().map_err(RecvError::from)?;
		self.framed.reader_mut().statistics.inc_total_items();

		self.deserializer
			.deserialize(&mut payload)
			.map_err(RecvError::Serde)
	}
}

impl<T, R, D> fmt::Debug for Receiver<T, R, D>
where
	R: fmt::Debug,
	D: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Receiver")
			.field("reader", self.framed.reader())
			.field("deserializer", &self.deserializer)
			.field("config", self.config())
			.finish_non_exhaustive()
	}
}

/// An iterator over received messages.
///
/// See: [`Receiver::incoming`].
#[derive(Debug)]
pub struct Incoming<'a, T, R, D> {
	receiver: &'a mut Receiver<T, R, D>,
}

impl<'a, T, R, D> Iterator for Incoming<'a, T, R, D>
where
	R: ReadExt,
	D: Deserializer<T>,
{
	type Item = Result<T, RecvError<D::Error, R::Error>>;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		Some(self.receiver.recv_blocking())
	}
}

impl<'a, T, R, D> Incoming<'a, T, R, D>
where
	R: AsyncReadExt + Unpin,
	D: Deserializer<T>,
{
	/// Return the next message.
	///
	/// This method is the async equivalent of [`Iterator::next()`].
	///
	/// # Cancel Safety
	///
	/// This method is cancel safe. If the method is used as the event in some
	/// `select!`-like macro and some other branch completes first, then it is
	/// guaranteed that no data will be dropped.
	#[inline]
	pub async fn next_async(
		&mut self,
	) -> Result<T, RecvError<D::Error, R::Error>> {
		self.receiver.recv().await
	}
}

/// A builder for [`Receiver`].
#[derive(Clone)]
#[must_use = "builders don't do anything unless you `.build()` them"]
pub struct Builder<T, R, D> {
	_marker: PhantomData<fn() -> T>,
	reader: R,
	deserializer: D,
	config: Option<Config>,
}

impl<T> Builder<T, (), ()> {
	/// Create a new [`Builder`] with the default options.
	///
	/// # Example
	///
	/// ```no_run
	/// let reader = std::io::empty();
	/// let deserializer = channels::serdes::Bincode::new();
	///
	/// let rx = channels::receiver::Builder::<i32, _, _>::new()
	///            .reader(reader)
	///            .deserializer(deserializer)
	///            .build();
	/// ```
	#[inline]
	pub fn new() -> Self {
		Builder {
			_marker: PhantomData,
			reader: (),
			deserializer: (),
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

impl<T, D> Builder<T, (), D> {
	/// Sets the reader of the [`Receiver`].
	///
	/// This function accepts both synchronous and asynchronous readers.
	///
	/// # Example
	///
	/// Synchronously:
	///
	/// ```no_run
	/// let builder = channels::Receiver::<i32, _, _>::builder()
	///                 .reader(std::io::empty());
	/// ```
	///
	/// Asynchronously:
	///
	/// ```no_run
	/// let builder = channels::Receiver::<i32, _, _>::builder()
	///                 .reader(tokio::io::empty());
	/// ```
	#[inline]
	pub fn reader<R>(
		self,
		reader: impl IntoRead<R>,
	) -> Builder<T, R, D> {
		Builder {
			_marker: PhantomData,
			reader: reader.into_read(),
			deserializer: self.deserializer,
			config: self.config,
		}
	}
}

impl<T, R> Builder<T, R, ()> {
	/// Sets the deserializer of the [`Receiver`].
	///
	/// # Example
	///
	/// ```no_run
	/// let deserializer = channels::serdes::Bincode::new();
	///
	/// let builder = channels::Receiver::<i32, _, _>::builder()
	///                 .deserializer(deserializer);
	/// ```
	#[inline]
	pub fn deserializer<D>(
		self,
		deserializer: D,
	) -> Builder<T, R, D> {
		Builder {
			_marker: PhantomData,
			reader: self.reader,
			deserializer,
			config: self.config,
		}
	}
}

impl<T, R, D> Builder<T, R, D> {
	/// Set the [`Config`] for this receiver.
	///
	/// # Example
	///
	/// ```no_run
	/// use channels::receiver::{Config, Receiver};
	///
	/// let config = Config::default()
	///                 .with_size_estimate(42);
	///
	/// let rx = Receiver::<i32, _, _>::builder()
	///             .config(config);
	/// ```
	#[inline]
	pub fn config(mut self, config: Config) -> Self {
		self.config = Some(config);
		self
	}

	/// Build a [`Receiver`].
	///
	/// # Example
	///
	/// ```no_run
	/// let rx: channels::Receiver<i32, _, _> = channels::Receiver::builder()
	///            .reader(std::io::empty())
	///            .deserializer(channels::serdes::Bincode::new())
	///            .build();
	/// ```
	#[inline]
	pub fn build(self) -> Receiver<T, R, D> {
		let Self { _marker, config, deserializer, reader } = self;

		Receiver {
			_marker: PhantomData,
			deserializer,
			framed: FramedRead::new(
				StatIO::new(reader),
				FrameDecoder {
					seq: FrameNumSequence::new(),
					config: config.unwrap_or_default(),
				},
			),
		}
	}
}

impl<T, R, D> fmt::Debug for Builder<T, R, D>
where
	R: fmt::Debug,
	D: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Builder")
			.field("reader", &self.reader)
			.field("deserializer", &self.deserializer)
			.field("config", &self.config)
			.finish()
	}
}
