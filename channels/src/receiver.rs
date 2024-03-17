//! Module containing the implementation for [`Receiver`].

use core::fmt;
use core::marker::PhantomData;

use crate::error::RecvError;
use crate::io::{AsyncRead, IntoReader, Read, Reader};
use crate::serdes::Deserializer;
use crate::util::{Pcb, StatIO, Statistics};

/// The receiving-half of the channel.
pub struct Receiver<T, R, D> {
	_marker: PhantomData<T>,
	reader: StatIO<R>,
	deserializer: D,
	pcb: Pcb,
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
	pub const fn builder() -> Builder<T, (), ()> {
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
	/// This function will return a future that will complete only when all the
	/// bytes of `T` have been received.
	pub async fn recv(
		&mut self,
	) -> Result<T, RecvError<D::Error, R::Error>> {
		let payload = crate::protocol::recv_async(
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
	/// This function will block the current thread until every last byte of
	/// `T` has been received.
	pub fn recv_blocking(
		&mut self,
	) -> Result<T, RecvError<D::Error, R::Error>> {
		let payload = crate::protocol::recv_sync(
			&mut self.pcb,
			&mut self.reader,
		)?;

		self.deserializer
			.deserialize(payload)
			.map_err(RecvError::Serde)
	}
}

impl<T, R, D> fmt::Debug for Receiver<T, R, D>
where
	StatIO<R>: fmt::Debug,
	D: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Receiver")
			.field("reader", &self.reader)
			.field("deserializer", &self.deserializer)
			.finish_non_exhaustive()
	}
}

unsafe impl<T, R, D> Send for Receiver<T, R, D>
where
	StatIO<R>: Send,
	D: Send,
{
}

unsafe impl<T, R, D> Sync for Receiver<T, R, D>
where
	StatIO<R>: Sync,
	D: Sync,
{
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
pub struct Builder<T, R, D> {
	_marker: PhantomData<T>,
	reader: R,
	deserializer: D,
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
			.finish()
	}
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
	pub const fn new() -> Self {
		Builder { _marker: PhantomData, reader: (), deserializer: () }
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
		}
	}
}

impl<T, R, D> Builder<T, R, D> {
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
		}
	}
}
