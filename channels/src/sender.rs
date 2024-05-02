//! Module containing the implementation for [`Sender`].

use core::borrow::Borrow;
use core::fmt;
use core::marker::PhantomData;

use crate::error::SendError;
use crate::io::{AsyncWrite, Container, IntoWrite, Write};
use crate::protocol::Pcb;
use crate::serdes::Serializer;
use crate::util::StatIO;

#[allow(unused_imports)]
use crate::util::Statistics;

/// The sending-half of the channel.
pub struct Sender<T, W, S> {
	_marker: PhantomData<fn() -> T>,
	writer: StatIO<W>,
	serializer: S,
	pcb: Pcb,
	config: Config,
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
	pub fn config(&self) -> &Config {
		&self.config
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
	pub fn get(&self) -> &W::Inner {
		self.writer.inner.get_ref()
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

	/// Destruct the sender and get back the underlying writer.
	pub fn into_writer(self) -> W::Inner {
		self.writer.inner.into_inner()
	}
}

impl<T, W, S> Sender<T, W, S>
where
	W: AsyncWrite,
	S: Serializer<T>,
{
	/// Attempts to send `data` through the channel.
	///
	/// # Example
	///
	/// ```no_run
	/// use tokio::net::TcpStream;
	///
	/// #[tokio::main]
	/// async fn main() {
	///     let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
	///     let mut tx = channels::Sender::<i32, _, _>::new(stream);
	///
	///     let data = 42;
	///
	///     tx.send(&data).await.unwrap();
	///     // or by taking ownership
	///     tx.send(data).await.unwrap();
	/// }
	/// ```
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

		crate::protocol::send_async(
			&self.config,
			&mut self.pcb,
			&mut self.writer,
			&payload,
		)
		.await?;

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
	/// Whether this function blocks execution is dependent on the underlying
	/// writer.
	///
	/// **NOTE:** Non-blocking writers (those who return `WouldBlock`) are _not_
	/// supported and will _not_ work. If you want non-blocking operation prefer
	/// the asynchronous version of this function, [`Sender::send()`].
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

		crate::protocol::send_sync(
			&self.config,
			&mut self.pcb,
			&mut self.writer,
			&payload,
		)?;

		Ok(())
	}
}

/// A builder for [`Sender`].
#[derive(Clone)]
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
	#[must_use]
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
	#[must_use]
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
	pub fn build(self) -> Sender<T, W, S> {
		Sender {
			_marker: PhantomData,
			writer: StatIO::new(self.writer),
			serializer: self.serializer,
			pcb: Pcb::new(),
			config: self.config.unwrap_or_default(),
		}
	}
}

/// Configuration for [`Sender`].
///
/// ## Flush on send
///
/// Flush the underlying writer after every [`send()`] or [`send_blocking()`].
///
/// **Default:** `true`
///
/// [`send()`]: Sender::send()
/// [`send_blocking()`]: Sender::send_blocking()
///
/// ## Use header checksum
///
/// Calculate the checksum for each packet before sending it.
///
/// This is only necessary when transmitting over unreliable transports.
/// Thus if you are using, say a UNIX socket, pipe, or anything that is
/// in-memory, you generally don't need this additional check. So it is
/// perfectly safe to turn off this extra compute if you find it to be a
/// bottleneck for your application.
///
/// **Default:** `true`
///
/// ## Coalesce writes
///
/// Coalesce packet writes into a single [`write()`] call.
///
/// Produced packets do not reside in contiguous chunks of memory. Because
/// [`write()`] expects a contiguous region of memory, this means that
/// writing the packet to the underlying writer is not as simple as calling
/// [`write()`] once.
///
/// In such situations, code can:
///
/// - A) call [`write()`] many times
/// - B) copy each part of the packet into a single contiguous region of
///      memory and call [`write()`] on it just once
///
/// Approach A has the benefit that it requires no additional allocations
/// or copying data. However, it heavily depends on the speed of the
/// underlying writer. Consider a writer that performs a system call each
/// time its [`write()`] method is called. Because system calls are
/// expensive, code should generally try to reduce their usage. For these
/// cases it is generally preferred to use approach B and only call [`write()`]
/// once. But in other cases where the writer is generally cheaply writable,
/// say writing to an in-memory buffer for further processing, it **can** be
/// **sometimes** beneficial to use approach A in terms of CPU time and memory
/// resources. Note that even in such cases, approach B might still be
/// faster.
///
/// This field basically controls the behavior of how [`write()`] is called.
/// Setting it to `true`, will signal the sending code to use approach B as
/// per the above. Setting it to `false`, will signal the sending code to
/// use approach A. Incorrectly setting this field, can lead to serious
/// performance issues. For this reason, the default behavior is approach B.
///
/// Writers should not assume anything about the pattern of how [`write()`]
/// is called.
///
/// **Default:** `true`
///
/// [`write()`]: Write::write()
#[derive(Clone)]
pub struct Config {
	flags: u8,
}

impl Config {
	const FLUSH_ON_SEND: u8 = 1 << 0;
	const USE_HEADER_CHECKSUM: u8 = 1 << 1;
	const COALESCE_WRITES: u8 = 1 << 2;

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
	fn default() -> Self {
		Self {
			flags: Self::FLUSH_ON_SEND
				| Self::USE_HEADER_CHECKSUM
				| Self::COALESCE_WRITES,
		}
	}
}

impl Config {
	/// Get whether the [`Sender`] will flush the writer after every send.
	#[must_use]
	pub fn flush_on_send(&self) -> bool {
		self.get_flag(Self::FLUSH_ON_SEND)
	}

	/// Whether the [`Sender`] will flush the writer after every send.
	pub fn set_flush_on_send(&mut self, yes: bool) -> &mut Self {
		self.set_flag(Self::FLUSH_ON_SEND, yes);
		self
	}

	/// Whether the [`Sender`] will flush the writer after every send.
	#[must_use]
	pub fn with_flush_on_send(mut self, yes: bool) -> Self {
		self.set_flush_on_send(yes);
		self
	}

	/// Get whether the [`Sender`] will calculate the checksum for sent packets.
	#[must_use]
	pub fn use_header_checksum(&self) -> bool {
		self.get_flag(Self::USE_HEADER_CHECKSUM)
	}

	/// Whether the [`Sender`] will calculate the checksum for sent packets.
	pub fn set_use_header_checksum(
		&mut self,
		yes: bool,
	) -> &mut Self {
		self.set_flag(Self::USE_HEADER_CHECKSUM, yes);
		self
	}

	/// Whether the [`Sender`] will calculate the checksum for sent packets.
	#[must_use]
	pub fn with_use_header_checksum(mut self, yes: bool) -> Self {
		self.set_use_header_checksum(yes);
		self
	}

	/// Get whether the [`Sender`] will coalesce all writes into a single buffer.
	#[must_use]
	pub fn coalesce_writes(&self) -> bool {
		self.get_flag(Self::COALESCE_WRITES)
	}

	/// Whether the [`Sender`] will coalesce all writes into a single buffer.
	pub fn set_coalesce_writes(&mut self, yes: bool) -> &mut Self {
		self.set_flag(Self::COALESCE_WRITES, yes);
		self
	}

	/// Whether the [`Sender`] will coalesce all writes into a single buffer.
	#[must_use]
	pub fn with_coalesce_writes(mut self, yes: bool) -> Self {
		self.set_coalesce_writes(yes);
		self
	}
}

impl fmt::Debug for Config {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Config")
			.field("flush_on_send", &self.flush_on_send())
			.field("use_header_checksum", &self.use_header_checksum())
			.field("coalesce_writes", &self.coalesce_writes())
			.finish()
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

impl<T, W, S> fmt::Debug for Sender<T, W, S>
where
	W: fmt::Debug,
	S: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Sender")
			.field("writer", &self.writer)
			.field("serializer", &self.serializer)
			.field("config", &self.config)
			.finish_non_exhaustive()
	}
}
