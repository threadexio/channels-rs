//! Module containing the implementation for [`Receiver`].

use core::fmt;
use core::marker::PhantomData;
use core::num::NonZeroUsize;

use crate::error::RecvError;
use crate::io::{AsyncRead, IntoReader, Read, Reader};
use crate::protocol::Pcb;
use crate::serdes::Deserializer;
use crate::util::StatIO;

#[allow(unused_imports)]
use crate::util::Statistics;

use channels_packet::PacketLength;

/// The receiving-half of the channel.
pub struct Receiver<T, R, D> {
	_marker: PhantomData<T>,
	reader: StatIO<R>,
	deserializer: D,
	pcb: Pcb,
	config: Config,
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
	#[must_use]
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
	pub fn new(reader: impl IntoReader<R>) -> Self {
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
	pub fn with_deserializer(
		reader: impl IntoReader<R>,
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
	/// use std::num::NonZeroUsize;
	///
	/// use channels::receiver::{Config, Receiver};
	/// use channels::serdes::Bincode;
	///
	/// let reader = std::io::empty();
	///
	/// let config = Config::default()
	///                 .size_estimate(NonZeroUsize::new(42).unwrap());
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
	pub fn config(&self) -> &Config {
		&self.config
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
	/// #[tokio::main]
	/// async fn main() {
	///     let reader = tokio::io::empty();
	///     let mut rx = channels::Receiver::<i32, _, _>::new(reader);
	///
	///     let mut incoming = rx.incoming();
	///
	///     loop {
	///         tokio::select! {
	///             message = incoming.next_async() => {
	///                 match message {
	///                     Ok(message) => println!("received: {message}"),
	///                     Err(err) => eprintln!("failed to receive message: {err}"),
	///                 }
	///             }
	///             // ...
	///         }
	///     }
	/// }
	/// ```
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
	#[cfg(feature = "statistics")]
	pub fn statistics(&self) -> &Statistics {
		&self.reader.statistics
	}
}

impl<T, R, D> Receiver<T, R, D>
where
	R: Reader,
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
	pub fn get(&self) -> &R::Inner {
		self.reader.inner.get()
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
	pub fn get_mut(&mut self) -> &mut R::Inner {
		self.reader.inner.get_mut()
	}
}

impl<T, R, D> Receiver<T, R, D>
where
	R: AsyncRead,
	D: Deserializer<T>,
{
	/// Attempts to receive a type `T` from the channel.
	///
	/// # Example
	///
	/// ```no_run
	/// use tokio::net::TcpStream;
	///
	/// #[tokio::main]
	/// async fn main() {
	///     let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
	///     let mut rx = channels::Receiver::<i32, _, _>::new(stream);
	///
	///     let received: i32 = rx.recv().await.unwrap();
	///     println!("{received}");
	/// }
	/// ```
	pub async fn recv(
		&mut self,
	) -> Result<T, RecvError<D::Error, R::Error>> {
		let payload = crate::protocol::recv_async(
			&self.config,
			&mut self.pcb,
			&mut self.reader,
		)
		.await?;

		self.deserializer
			.deserialize(payload)
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
	/// Whether this function blocks execution is dependent on the underlying
	/// reader.
	///
	/// **NOTE:** Non-blocking readers (those who return `WouldBlock`) are _not_
	/// supported and will _not_ work. If you want non-blocking operation prefer
	/// the asynchronous version of this function, [`Receiver::recv()`].
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
	pub fn recv_blocking(
		&mut self,
	) -> Result<T, RecvError<D::Error, R::Error>> {
		let payload = crate::protocol::recv_sync(
			&self.config,
			&mut self.pcb,
			&mut self.reader,
		)?;

		self.deserializer
			.deserialize(payload)
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
	R: Read,
	D: Deserializer<T>,
{
	type Item = Result<T, RecvError<D::Error, R::Error>>;

	fn next(&mut self) -> Option<Self::Item> {
		Some(self.receiver.recv_blocking())
	}
}

impl<'a, T, R, D> Incoming<'a, T, R, D>
where
	R: AsyncRead,
	D: Deserializer<T>,
{
	/// Return the next message.
	///
	/// This method is the async equivalent of [`Iterator::next()`].
	pub async fn next_async(
		&mut self,
	) -> Result<T, RecvError<D::Error, R::Error>> {
		self.receiver.recv().await
	}
}

/// A builder for [`Receiver`].
#[derive(Clone)]
pub struct Builder<T, R, D> {
	_marker: PhantomData<T>,
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
	#[must_use]
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
	pub fn reader<R>(
		self,
		reader: impl IntoReader<R>,
	) -> Builder<T, R, D> {
		Builder {
			_marker: PhantomData,
			reader: reader.into_reader(),
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
	/// use core::num::NonZeroUsize;
	///
	/// use channels::receiver::{Config, Receiver};
	///
	/// let config = Config::default()
	///                 .size_estimate(NonZeroUsize::new(42).unwrap());
	///
	/// let rx = Receiver::<i32, _, _>::builder()
	///             .config(config);
	/// ```
	#[must_use]
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
	pub fn build(self) -> Receiver<T, R, D> {
		Receiver {
			_marker: PhantomData,
			reader: StatIO::new(self.reader),
			deserializer: self.deserializer,
			pcb: Pcb::new(),
			config: self.config.unwrap_or_default(),
		}
	}
}

/// Configuration for [`Receiver`].
///
/// [`Receiver`]: crate::Receiver
#[derive(Clone)]
pub struct Config {
	pub(crate) size_estimate: Option<NonZeroUsize>,
	pub(crate) max_size: usize,
	pub(crate) verify_header_checksum: bool,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			size_estimate: None,
			max_size: PacketLength::MAX.as_usize(),
			verify_header_checksum: true,
		}
	}
}

impl Config {
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
	#[must_use]
	pub fn size_estimate(mut self, estimate: NonZeroUsize) -> Self {
		self.size_estimate = Some(estimate);
		self
	}

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
	/// the receiver operation will return with an error of
	/// [`RecvError::Protocol(ProtocolError::ExceededMaximumSize)`].
	///
	/// This attack is only possible if malicious actors are able to
	/// talk directly with the [`Receiver`]. For example, if there is an
	/// encrypted and trusted channel between the receiver and the sender, then
	/// this attack is not applicable.
	///
	/// **Default:** 65K
	///
	/// [`recv()`]: Receiver::recv()
	/// [`recv_blocking()`]: Receiver::recv_blocking()
	/// [`send()`]: crate::Sender::send()
	/// [`send_blocking()`]: crate::Sender::send_blocking()
	/// [IP Fragmentation]: https://en.wikipedia.org/wiki/Internet_Protocol_version_4#Fragmentation
	/// [`RecvError::Protocol(ProtocolError::ExceededMaximumSize)`]: crate::error::ProtocolError::ExceededMaximumSize
	#[must_use]
	pub fn max_size(mut self, max_size: usize) -> Self {
		self.max_size = max_size;
		self
	}

	/// Verify the header checksum of each received packet.
	///
	/// This should be paired with a [`Sender`] that also does not produce
	/// checksums (see [`use_header_checksum()`]).
	///
	/// **Default:** `true`
	///
	/// [`Sender`]: crate::Sender
	/// [`use_header_checksum()`]: crate::sender::Config::use_header_checksum()
	#[must_use]
	pub fn verify_header_checksum(mut self, yes: bool) -> Self {
		self.verify_header_checksum = yes;
		self
	}
}

impl fmt::Debug for Config {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Config")
			.field("size_estimate", &self.size_estimate)
			.field("max_size", &self.max_size)
			.field(
				"verify_header_checksum",
				&self.verify_header_checksum,
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
			.field("reader", &self.reader)
			.field("deserializer", &self.deserializer)
			.field("config", &self.config)
			.finish_non_exhaustive()
	}
}

unsafe impl<T, R: Send, D: Send> Send for Builder<T, R, D> {}
unsafe impl<T, R: Sync, D: Sync> Sync for Builder<T, R, D> {}

unsafe impl<T, R: Send, D: Send> Send for Receiver<T, R, D> {}
unsafe impl<T, R: Sync, D: Sync> Sync for Receiver<T, R, D> {}
