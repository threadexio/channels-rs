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

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
#[derive(Clone)]
pub struct Receiver<T, R, D> {
	_marker: PhantomData<T>,
	reader: Reader<R>,
	deserializer: D,
	pcb: Pcb,
}

impl<T> Receiver<T, (), ()> {
	/// Create a new builder.
	pub const fn builder() -> Builder<T, (), ()> {
		Builder::new()
	}
}

#[cfg(feature = "bincode")]
impl<T, R> Receiver<T, R, crate::serdes::Bincode> {
	/// Creates a new [`Receiver`] from `reader`.
	pub fn new(reader: R) -> Self {
		Self::with_deserializer(reader, Default::default())
	}
}

impl<T, R, D> Receiver<T, R, D> {
	/// Create a new [`Receiver`] from `reader` that uses `deserializer`.
	pub fn with_deserializer(reader: R, deserializer: D) -> Self {
		Receiver::builder()
			.reader(reader)
			.deserializer(deserializer)
			.build()
	}

	/// Get a reference to the underlying reader.
	pub fn get(&self) -> &R {
		&self.reader.inner
	}

	/// Get a mutable reference to the underlying writer. Directly writing to
	/// the stream is not advised.
	pub fn get_mut(&mut self) -> &mut R {
		&mut self.reader.inner
	}

	/// Get an iterator over incoming messages.
	pub fn incoming(&mut self) -> Incoming<'_, T, R, D> {
		Incoming { receiver: self }
	}

	/// Get statistics on this receiver.
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

unsafe impl<T, R: Send, D: Send> Send for Receiver<T, R, D> {}
unsafe impl<T, R: Sync, D: Sync> Sync for Receiver<T, R, D> {}

/// An iterator over received messages.
#[derive(Debug)]
pub struct Incoming<'a, T, R, D> {
	receiver: &'a mut Receiver<T, R, D>,
}

/// A builder that when completed will return a [`Receiver`].
#[derive(Debug)]
pub struct Builder<T, R, D> {
	_marker: PhantomData<T>,
	reader: R,
	deserializer: D,
}

impl<T> Builder<T, (), ()> {
	/// Create a new [`Builder`] with the default options.
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
	/// Use this synchronous reader.
	pub fn reader<R>(self, reader: R) -> Builder<T, R, D> {
		Builder {
			_marker: PhantomData,
			reader,
			deserializer: self.deserializer,
		}
	}
}

impl<T, R> Builder<T, R, ()> {
	/// Use this deserializer.
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
	/// Finalize the builder and build a [`Receiver`]
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
		use Poll::*;

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
					match ready!(read_all(self.reader, part))
						.map_err(RecvPayloadError::Io)
					{
						Err(e) => {
							return Ready(Err(e));
						},
						Ok(_) => {
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
					}
				},
				State::PartialPayload { ref header } => match ready!(
					read_all(self.reader, &mut self.payload)
				)
				.map_err(RecvPayloadError::Io)
				{
					Err(e) => {
						return Ready(Err(e));
					},
					Ok(_) if header.flags & Flags::MORE_DATA => {
						self.state = State::INITIAL;
					},
					Ok(_) => {
						let payload =
							core::mem::take(self.payload.inner_mut());
						return Ready(Ok(payload));
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
	use super::*;

	use core::future::Future;
	use core::pin::{pin, Pin};
	use core::task::Context;

	#[cfg(feature = "tokio")]
	mod imp {
		use super::*;

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
			use Poll::*;

			while buf.has_remaining() {
				let mut read_buf =
					io::ReadBuf::new(buf.unfilled_mut());
				let l0 = read_buf.filled().len();
				match ready!(pin!(&mut reader.inner)
					.poll_read(cx, &mut read_buf))
				{
					Err(e) if e.kind() == E::Interrupted => continue,
					Err(e) if e.kind() == E::WouldBlock => {
						return Poll::Pending
					},
					Err(e) => return Ready(Err(e)),
					Ok(_) => {
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
						return Poll::Pending
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
	use super::*;

	use crate::util::PollExt;

	mod imp {
		use super::*;

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
			use Poll::*;

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
