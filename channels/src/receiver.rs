//! Module containing the implementation for [`Receiver`].

use core::fmt;
use core::marker::PhantomData;
use core::num::NonZeroUsize;

use channels_packet::codec::FrameDecoder;

use crate::error::RecvError;
use crate::io::framed::FramedRead;
use crate::io::source::{AsyncSource, Source};
use crate::io::{AsyncReadExt, Container, IntoRead, ReadExt};
use crate::serdes::Deserializer;

#[allow(unused_imports)]
use crate::statistics::{StatIO, Statistics};

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
	R: AsyncReadExt + Unpin,
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
		self.framed.reader_mut().statistics.inc_ops();

		self.deserializer
			.deserialize(&mut payload)
			.map_err(RecvError::Serde)
	}
}

impl<T, R, D> Receiver<T, R, D>
where
	R: ReadExt,
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
		self.framed.reader_mut().statistics.inc_ops();

		self.deserializer
			.deserialize(&mut payload)
			.map_err(RecvError::Serde)
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

		// TODO: config
		let _ = config;
		let reader = StatIO::new(reader);
		let decoder = FrameDecoder::new();
		let framed = FramedRead::new(reader, decoder);

		Receiver { _marker: PhantomData, deserializer, framed }
	}
}

/// Configuration for [`Receiver`].
///
/// ## Size estimate
///
/// Size estimate for incoming data.
///
/// Inform the receiving code to preallocate a buffer of this size when
/// receiving. Setting this field correctly can help avoid reallocations
/// when receiving data that is split up into multiple packets. For most
/// cases, when data fits inside a single packet, this field has _no_ effect
/// and can simply be left as the default.
///
/// When setting this field, if you don't know the exact size of your data,
/// it is best to overestimate it. Setting a value even one byte less than
/// what the actual size of the data is will still lead to a reallocation
/// and more copying. However if the data fits inside one packet, as is with
/// most cases, setting this field incorrectly can still have a minor impact
/// on performance.
///
/// In general, this field should probably be left alone unless you can
/// prove that the processing time for received packets far exceeds the
/// transmission time of the medium used.
///
/// **NOTE:** Setting this field to `0` disables any preallocations.
///
/// **Default:** `0`
///
/// [`Receiver`]: crate::Receiver
///
/// ## Max size
///
/// Maximum size of data each call to [`recv()`] or [`recv_blocking()`] will
/// read.
///
/// In order for large payloads to be transmitted, they have to be split up
/// to multiple packets. Packets contain a flag for the receiver to wait for
/// more packets if the payload was not fully sent in that packet (because
/// it had to be split up). This way of transmitting large payloads is also
/// used in IPv4 and it is called [IPv4 Fragmentation].
///
/// Because the receiver does not know the full length from the first packet,
/// it can only know this once all packets have been received and their
/// lengths are summed, it doesn't know how much memory it will need to hold
/// the full payload. For this reason, it is possible for a malicious actor
/// to send keep sending carefully crafted packets all with that flag set,
/// until the receiver exhausts all their memory. This DOS attack is the
/// reason for the existence of this option.
///
/// The default value allows payloads up to 65K and it should be enough for
/// almost all cases. Unless you are sending over 65K of data in one [`send()`]
/// or [`send_blocking()`], the default value is perfectly safe. Please note
/// that if you do send more than 65K, you will have to set this field to
/// fit your needs.
///
/// This field can also make the system more secure. For example, if you
/// know in advance the maximum length one payload can take up, you should
/// set this field to limit wasted memory by bad actors.
///
/// If it happens and a receiver does read more than the configured limit,
/// the receiver operation will return with an error of [`RecvError::ExceededMaximumSize`].
///
/// This attack is only possible if malicious actors are able to
/// talk directly with the [`Receiver`]. For example, if there is an
/// encrypted and trusted channel between the receiver and the sender, then
/// this attack is not applicable.
///
/// Setting this field to `0` disables this mechanism and allows payloads
/// of any size.
///
/// **Default:** 65K
///
/// [`recv()`]: Receiver::recv()
/// [`recv_blocking()`]: Receiver::recv_blocking()
/// [`send()`]: crate::Sender::send()
/// [`send_blocking()`]: crate::Sender::send_blocking()
/// [IP Fragmentation]: https://en.wikipedia.org/wiki/Internet_Protocol_version_4#Fragmentation
///
/// ## Verify header checksum
///
/// Verify the header checksum of each received packet.
///
/// This should be paired with a [`Sender`] that also does not produce
/// checksums (see [`use_header_checksum()`]).
///
/// **Default:** `true`
///
/// [`Sender`]: crate::Sender
/// [`use_header_checksum()`]: crate::sender::Config::use_header_checksum()
///
/// ## Verify packet order
///
/// Verify that received packets are in order.
///
/// Using the library atop of mediums which do not guarantee any sort of ordering
/// between packets, such as UDP, can present some problems. Each channels packet
/// contains an wrapping numeric ID that is used to check whether packets are
/// received in the order they were sent. UDP for example, does not guarantee in
/// which order packets reach their destination. Supposing a sender tries to send
/// packets with IDs 1, 2 and 3, it is therefore possible that UDP delivers packets
/// 2 or 3 before 1. This would immediately trigger an error of [`OutOfOrder`].
/// This behavior might not be what you want. This flag specifies whether the
/// receiver should check the ID of each packet and verify that it was received
/// in the correct order. If it is not set, then it is impossible for an [`OutOfOrder`]
/// error to occur.
///
/// [`OutOfOrder`]: RecvError::OutOfOrder
#[derive(Clone)]
#[must_use = "`Config`s don't do anything on their own"]
pub struct Config {
	pub(crate) size_estimate: Option<NonZeroUsize>,
	pub(crate) max_size: Option<NonZeroUsize>,
	pub(crate) flags: u8,
}

impl Config {
	const VERIFY_HEADER_CHECKSUM: u8 = 1 << 0;
	const VERIFY_PACKET_ORDER: u8 = 1 << 1;

	#[inline]
	fn get_flag(&self, flag: u8) -> bool {
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
		Self {
			size_estimate: None,
			max_size: None,
			flags: Self::VERIFY_PACKET_ORDER
				| Self::VERIFY_HEADER_CHECKSUM,
		}
	}
}

impl Config {
	/// Get the size estimate of the [`Receiver`].
	#[inline]
	#[must_use]
	pub fn size_estimate(&self) -> usize {
		self.size_estimate.map_or(0, NonZeroUsize::get)
	}

	/// Set the size estimate of the [`Receiver`].
	#[allow(clippy::missing_panics_doc)]
	#[inline]
	pub fn set_size_estimate(
		&mut self,
		estimate: usize,
	) -> &mut Self {
		self.size_estimate = match estimate {
			0 => None,
			x => Some(
				NonZeroUsize::new(x)
					.expect("size_estimate should never be 0"),
			),
		};
		self
	}

	/// Set the size estimate of the [`Receiver`].
	#[inline]
	pub fn with_size_estimate(mut self, estimate: usize) -> Self {
		self.set_size_estimate(estimate);
		self
	}

	/// Get the max size of the [`Receiver`].
	#[inline]
	#[must_use]
	pub fn max_size(&self) -> usize {
		self.max_size.map_or(0, NonZeroUsize::get)
	}

	/// Set the max size of the [`Receiver`].
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

	/// Set the max size of the [`Receiver`].
	#[inline]
	pub fn with_max_size(mut self, max_size: usize) -> Self {
		self.set_max_size(max_size);
		self
	}

	/// Get whether the [`Receiver`] will verify each packet's header with the
	/// checksum.
	#[inline]
	#[must_use]
	pub fn verify_header_checksum(&self) -> bool {
		self.get_flag(Self::VERIFY_HEADER_CHECKSUM)
	}

	/// Whether to verify each packet's header with the checksum.
	#[inline]
	pub fn set_verify_header_checksum(
		&mut self,
		yes: bool,
	) -> &mut Self {
		self.set_flag(Self::VERIFY_HEADER_CHECKSUM, yes);
		self
	}

	/// Whether to verify each packet's header with the checksum.
	#[inline]
	pub fn with_verify_header_checksum(mut self, yes: bool) -> Self {
		self.set_verify_header_checksum(yes);
		self
	}

	/// Get whether to verify packet order.
	#[inline]
	#[must_use]
	pub fn verify_packet_order(&self) -> bool {
		self.get_flag(Self::VERIFY_PACKET_ORDER)
	}

	/// Whether to verify packet order.
	#[inline]
	pub fn set_verify_packet_order(
		&mut self,
		yes: bool,
	) -> &mut Self {
		self.set_flag(Self::VERIFY_PACKET_ORDER, yes);
		self
	}

	/// Whether to verify packet order.
	#[inline]
	pub fn with_verify_packet_order(mut self, yes: bool) -> Self {
		self.set_verify_packet_order(yes);
		self
	}
}

impl fmt::Debug for Config {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Config")
			.field("size_estimate", &self.size_estimate())
			.field("max_size", &self.max_size())
			.field(
				"verify_header_checksum",
				&self.verify_header_checksum(),
			)
			.finish()
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
