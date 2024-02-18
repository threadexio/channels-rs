//! Module containing the implementation for [`Receiver`].

use core::fmt;
use core::marker::PhantomData;
use core::task::{ready, Poll};

use alloc::vec::Vec;

use channels_packet::{Flags, Header};
use channels_serdes::Deserializer;

#[allow(unused_imports)]
use crate::common::{Pcb, Statistics};
use crate::error::{RecvError, VerifyError};
use crate::util::{BufMut, IoSlice};

/// The receiving-half of the channel.
pub struct Receiver<T, R, D> {
	_marker: PhantomData<T>,
	reader: Reader<R>,
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
	pub fn new(reader: R) -> Self {
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
	pub fn with_deserializer(reader: R, deserializer: D) -> Self {
		Receiver::builder()
			.reader(reader)
			.deserializer(deserializer)
			.build()
	}

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
	pub fn get(&self) -> &R {
		&self.reader.inner
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
	pub fn get_mut(&mut self) -> &mut R {
		&mut self.reader.inner
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

impl<T, R, D> fmt::Debug for Receiver<T, R, D>
where
	Reader<R>: fmt::Debug,
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
	Reader<R>: Send,
	D: Send,
{
}

unsafe impl<T, R, D> Sync for Receiver<T, R, D>
where
	Reader<R>: Sync,
	D: Sync,
{
}

/// An iterator over received messages.
#[derive(Debug)]
pub struct Incoming<'a, T, R, D> {
	receiver: &'a mut Receiver<T, R, D>,
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
	pub fn reader<R>(self, reader: R) -> Builder<T, R, D> {
		Builder {
			_marker: PhantomData,
			reader,
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
			reader: Reader::new(self.reader),
			deserializer: self.deserializer,
			pcb: Pcb::new(),
		}
	}
}

#[derive(Debug, Clone)]
struct Reader<R> {
	inner: R,

	#[cfg(feature = "statistics")]
	statistics: Statistics,
}

impl<R> Reader<R> {
	pub const fn new(reader: R) -> Self {
		Self {
			inner: reader,

			#[cfg(feature = "statistics")]
			statistics: Statistics::new(),
		}
	}

	#[allow(unused_variables)]
	fn on_read(&mut self, n: u64) {
		#[cfg(feature = "statistics")]
		self.statistics.add_total_bytes(n);
	}
}

enum State {
	PartialHeader { part: IoSlice<[u8; Header::SIZE]> },
	PartialPayload { header: Header },
}

impl State {
	pub const INITIAL: State = State::PartialHeader {
		part: IoSlice::new([0u8; Header::SIZE]),
	};
}

enum RecvPayloadError<E> {
	Io(E),
	Verify(VerifyError),
}

impl<Ser, Io> From<RecvPayloadError<Io>> for RecvError<Ser, Io> {
	fn from(value: RecvPayloadError<Io>) -> Self {
		use RecvError as R;
		use RecvPayloadError as L;

		match value {
			L::Io(e) => R::Io(e),
			L::Verify(e) => R::Verify(e),
		}
	}
}

struct RecvPayload<'a, R> {
	reader: &'a mut Reader<R>,
	pcb: &'a mut Pcb,
	payload: IoSlice<Vec<u8>>,
	state: State,
}

impl<'a, R> RecvPayload<'a, R> {
	fn new(pcb: &'a mut Pcb, reader: &'a mut Reader<R>) -> Self {
		Self {
			pcb,
			reader,
			state: State::INITIAL,
			payload: IoSlice::new(Vec::new()),
		}
	}

	fn advance<F, E>(
		&mut self,
		mut read_all: F,
	) -> Poll<Result<Vec<u8>, RecvPayloadError<E>>>
	where
		F: FnMut(
			&mut Reader<R>,
			&mut dyn BufMut,
		) -> Poll<Result<(), E>>,
	{
		use Poll::Ready;

		/// Grow `vec` by `len` bytes and return the newly-allocated bytes as a slice.
		fn reserve_slice_in_vec(
			vec: &mut Vec<u8>,
			len: usize,
		) -> &mut [u8] {
			let start = vec.len();
			let new_len = usize::saturating_add(start, len);
			vec.resize(new_len, 0);
			&mut vec[start..new_len]
		}

		loop {
			match self.state {
				State::PartialHeader { ref mut part } => {
					match ready!(read_all(self.reader, part)) {
						Ok(()) => {
							let header = Header::read_from(
								part.inner_ref(),
								&mut self.pcb.id_gen,
							)
							.map_err(Into::into)
							.map_err(RecvPayloadError::Verify)?;

							reserve_slice_in_vec(
								self.payload.inner_mut(),
								header
									.length
									.to_payload_length()
									.as_usize(),
							);

							self.state =
								State::PartialPayload { header };
						},
						Err(e) => {
							return Ready(Err(RecvPayloadError::Io(
								e,
							)));
						},
					}
				},
				State::PartialPayload { ref header } => match ready!(
					read_all(self.reader, &mut self.payload)
				) {
					Ok(()) => {
						#[cfg(feature = "statistics")]
						self.reader.statistics.inc_packets();

						if header.flags & Flags::MORE_DATA {
							self.state = State::INITIAL;
						} else {
							let payload = core::mem::take(
								self.payload.inner_mut(),
							);

							#[cfg(feature = "statistics")]
							self.reader.statistics.inc_ops();

							return Ready(Ok(payload));
						}
					},
					Err(e) => {
						return Ready(Err(RecvPayloadError::Io(e)));
					},
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
		ready, BufMut, Deserializer, Incoming, Poll, Reader,
		Receiver, RecvError, RecvPayload, RecvPayloadError, Vec,
	};

	use core::future::Future;
	use core::pin::{pin, Pin};
	use core::task::Context;

	#[cfg(feature = "tokio")]
	mod imp {
		use super::{pin, ready, BufMut, Context, Poll, Reader};

		use tokio::io;

		pub use io::{AsyncRead, Error};

		pub fn poll_read<R>(
			reader: &mut Reader<R>,
			buf: &mut dyn BufMut,
			cx: &mut Context,
		) -> Poll<Result<(), Error>>
		where
			R: AsyncRead + Unpin,
		{
			use io::ErrorKind as E;
			use Poll::{Pending, Ready};

			while buf.has_remaining() {
				let mut read_buf =
					io::ReadBuf::new(buf.unfilled_mut());
				let l0 = read_buf.filled().len();
				match ready!(pin!(&mut reader.inner)
					.poll_read(cx, &mut read_buf))
				{
					Err(e) if e.kind() == E::Interrupted => continue,
					Err(e) if e.kind() == E::WouldBlock => {
						return Pending
					},
					Err(e) => return Ready(Err(e)),
					Ok(()) => {
						let l1 = read_buf.filled().len();
						let dl = l1 - l0;

						if dl != 0 {
							buf.advance(dl);
							reader.on_read(dl as u64);
						} else {
							return Ready(Err(
								E::UnexpectedEof.into()
							));
						}
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

		pub use futures::AsyncRead;
		pub use io::Error;

		pub fn poll_read<R>(
			reader: &mut Reader<R>,
			buf: &mut dyn BufMut,
			cx: &mut Context,
		) -> Poll<io::Result<()>>
		where
			R: AsyncRead + Unpin,
		{
			use io::ErrorKind as E;
			use Poll::*;

			while buf.has_remaining() {
				match ready!(pin!(&mut reader.inner)
					.poll_read(cx, buf.unfilled_mut()))
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
						buf.advance(n);
						reader.on_read(n as u64);
					},
				}
			}

			Ready(Ok(()))
		}
	}

	use self::imp::{poll_read, AsyncRead, Error};

	impl<R> Future for RecvPayload<'_, R>
	where
		R: AsyncRead + Unpin,
	{
		type Output = Result<Vec<u8>, RecvPayloadError<Error>>;

		fn poll(
			mut self: Pin<&mut Self>,
			cx: &mut Context<'_>,
		) -> Poll<Self::Output> {
			self.advance(|r, b| poll_read(r, b, cx))
		}
	}

	impl<T, R, D> Receiver<T, R, D>
	where
		R: AsyncRead + Unpin,
		D: Deserializer<T>,
	{
		/// Attempts to receive a type `T` from the channel.
		///
		/// This function will return a future that will complete only when all the
		/// bytes of `T` have been received.
		pub async fn recv(
			&mut self,
		) -> Result<T, RecvError<D::Error, Error>> {
			let mut payload =
				RecvPayload::new(&mut self.pcb, &mut self.reader)
					.await?;

			self.deserializer
				.deserialize(&mut payload)
				.map_err(RecvError::Serde)
		}
	}

	impl<'a, T, R, D> Incoming<'a, T, R, D>
	where
		R: AsyncRead + Unpin,
		D: Deserializer<T>,
	{
		/// Return the next message.
		///
		/// This method is the async equivalent of [`Iterator::next()`].
		pub async fn next_async(
			&mut self,
		) -> Result<T, RecvError<D::Error, Error>> {
			self.receiver.recv().await
		}
	}
}

mod sync_impl {
	use super::{
		BufMut, Deserializer, Incoming, Poll, Reader, Receiver,
		RecvError, RecvPayload,
	};

	use crate::util::PollExt;

	mod imp {
		use super::{BufMut, Poll, Reader};

		use std::io;

		pub use io::{Error, Read};

		pub fn read<R>(
			reader: &mut Reader<R>,
			buf: &mut dyn BufMut,
		) -> Poll<io::Result<()>>
		where
			R: Read,
		{
			use io::ErrorKind as E;
			use Poll::{Pending, Ready};

			while buf.has_remaining() {
				match reader.inner.read(buf.unfilled_mut()) {
					Err(e) if e.kind() == E::Interrupted => continue,
					Err(e) if e.kind() == E::WouldBlock => {
						return Pending
					},
					Err(e) => return Ready(Err(e)),
					Ok(0) => {
						return Ready(Err(E::UnexpectedEof.into()))
					},
					Ok(n) => {
						buf.advance(n);
						reader.on_read(n as u64);
					},
				}
			}

			Ready(Ok(()))
		}
	}

	use self::imp::{read, Error, Read};

	impl<T, R, D> Receiver<T, R, D>
	where
		R: Read,
		D: Deserializer<T>,
	{
		/// Attempts to receive a type `T` from the channel.
		///
		/// This function will block the current thread until every last byte of
		/// `T` has been received.
		///
		/// # Panics
		///
		/// Panics if the underlying reader returns with `WouldBlock`.
		#[track_caller]
		pub fn recv_blocking(
			&mut self,
		) -> Result<T, RecvError<D::Error, Error>> {
			let mut payload =
				RecvPayload::new(&mut self.pcb, &mut self.reader)
					.advance(read)
					.unwrap()?;

			self.deserializer
				.deserialize(&mut payload)
				.map_err(RecvError::Serde)
		}
	}

	impl<'a, T, R, D> Iterator for Incoming<'a, T, R, D>
	where
		R: Read,
		D: Deserializer<T>,
	{
		type Item = Result<T, RecvError<D::Error, Error>>;

		fn next(&mut self) -> Option<Self::Item> {
			Some(self.receiver.recv_blocking())
		}
	}
}
