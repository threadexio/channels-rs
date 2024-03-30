//! Module containing the implementation for [`Sender`].

use core::borrow::Borrow;
use core::fmt;
use core::marker::PhantomData;

use crate::error::SendError;
use crate::io::{AsyncWrite, IntoWriter, Write, Writer};
use crate::protocol::Pcb;
use crate::serdes::Serializer;
use crate::util::StatIO;

#[allow(unused_imports)]
use crate::util::Statistics;

/// The sending-half of the channel.
pub struct Sender<T, W, S> {
	_marker: PhantomData<T>,
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
	///                 .flush_on_send(false);
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
			payload,
		)
		.await
		.map_err(SendError::Io)?;

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
			payload,
		)
		.map_err(SendError::Io)?;

		Ok(())
	}
}

/// A builder for [`Sender`].
#[derive(Clone)]
pub struct Builder<T, W, S> {
	_marker: PhantomData<T>,
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
		writer: impl IntoWriter<W>,
	) -> Builder<T, W, S> {
		Builder {
			_marker: PhantomData,
			writer: writer.into_writer(),
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
	///                 .flush_on_send(false);
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
#[derive(Clone)]
pub struct Config {
	pub(crate) flush_on_send: bool,
	pub(crate) use_header_checksum: bool,
}

impl Default for Config {
	fn default() -> Self {
		Self { flush_on_send: true, use_header_checksum: true }
	}
}

impl Config {
	/// Flush the underlying writer after every [`send()`] or [`send_blocking()`].
	///
	/// **Default:** `true`
	///
	/// [`send()`]: Sender::send()
	/// [`send_blocking()`]: Sender::send_blocking()
	#[must_use]
	pub fn flush_on_send(mut self, yes: bool) -> Self {
		self.flush_on_send = yes;
		self
	}

	/// Calculate the checksum for each packet before sending it.
	///
	/// This is only necessary when transmitting over unreliable transports.
	/// Thus if you are using, say a UNIX socket, pipe, or anything that is
	/// in-memory, you generally don't need this additional check. So it is
	/// perfectly safe to turn off this extra compute if you find it to be a
	/// bottleneck if you application.
	///
	/// **Default:** `true`
	#[must_use]
	pub fn use_header_checksum(mut self, yes: bool) -> Self {
		self.use_header_checksum = yes;
		self
	}
}

impl fmt::Debug for Config {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Config")
			.field("flush_on_send", &self.flush_on_send)
			.field("use_header_checksum", &self.use_header_checksum)
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
			.field("writer", &self.writer)
			.field("serializer", &self.serializer)
			.field("config", &self.config)
			.finish()
	}
}

impl<T, W, S> fmt::Debug for Sender<T, W, S>
where
	StatIO<W>: fmt::Debug,
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

unsafe impl<T, W: Send, S: Send> Send for Builder<T, W, S> {}
unsafe impl<T, W: Sync, S: Sync> Sync for Builder<T, W, S> {}

unsafe impl<T, W: Send, S: Send> Send for Sender<T, W, S> {}
unsafe impl<T, W: Sync, S: Sync> Sync for Sender<T, W, S> {}
