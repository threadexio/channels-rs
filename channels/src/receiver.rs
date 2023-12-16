//! Module containing the implementation for [`Receiver`].

use core::marker::PhantomData;
use core::mem::take;
use core::task::ready;
use core::task::Poll;

use alloc::vec::Vec;

use channels_io::{BufMut, IoSlice, PollExt};

use channels_packet::{Flags, Header};
use channels_serdes::Deserializer;

#[allow(unused_imports)]
use crate::common::{Pcb, Statistics};
use crate::error::{RecvError, VerifyError};

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
#[derive(Debug, Clone)]
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

impl<T, R, D> Receiver<T, R, D> {
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
	pub fn deserializer<D>(self, deserializer: D) -> Builder<T, R, D>
	where
		D: Deserializer<T>,
	{
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

struct Recv<'a, R> {
	reader: &'a mut Reader<R>,
	pcb: &'a mut Pcb,
	payload: IoSlice<Vec<u8>>,
	state: State,
}

impl<'a, R> Recv<'a, R> {
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
						let payload = take(self.payload.inner_mut());
						return Ready(Ok(payload));
					},
				},
			}
		}
	}
}

/// Grow `vec` by `len` bytes and return the newly-allocated bytes as a slice.
fn reserve_slice_in_vec(vec: &mut Vec<u8>, len: usize) -> &mut [u8] {
	let start = vec.len();
	let new_len = usize::saturating_add(start, len);
	vec.resize(new_len, 0);
	&mut vec[start..new_len]
}

#[cfg(feature = "std")]
mod std_impl {
	use super::*;

	use std::io::{self, Read};

	impl<R> Reader<R>
	where
		R: Read,
	{
		pub fn read_std(
			&mut self,
			buf: &mut dyn BufMut,
		) -> Poll<io::Result<()>> {
			use io::ErrorKind as E;
			use Poll::*;

			while buf.has_remaining() {
				match self.inner.read(buf.unfilled_mut()) {
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
						self.on_read(n as u64);
					},
				}
			}

			Ready(Ok(()))
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
		///
		/// # Panics
		///
		/// Panics if the underlying reader returns with `WouldBlock`.
		#[track_caller]
		pub fn recv_blocking(
			&mut self,
		) -> Result<T, RecvError<D::Error, io::Error>> {
			let mut payload =
				Recv::new(&mut self.pcb, &mut self.reader)
					.advance(|r, buf| r.read_std(buf))
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
		type Item = Result<T, RecvError<D::Error, io::Error>>;

		fn next(&mut self) -> Option<Self::Item> {
			Some(self.receiver.recv_blocking())
		}
	}
}

#[cfg(all(feature = "tokio", feature = "futures"))]
core::compile_error!(
	"tokio and futures features cannot be active at the same time"
);

#[cfg(feature = "tokio")]
#[cfg(not(feature = "futures"))]
mod tokio_impl {
	use super::*;

	use core::future::Future;
	use core::pin::{pin, Pin};
	use core::task::Context;

	use tokio::io::{self, AsyncRead};

	impl<R> Reader<R>
	where
		R: AsyncRead + Unpin,
	{
		pub fn poll_read_tokio(
			&mut self,
			cx: &mut Context,
			buf: &mut dyn BufMut,
		) -> Poll<io::Result<()>> {
			use io::ErrorKind as E;
			use Poll::*;

			while buf.has_remaining() {
				let mut read_buf =
					io::ReadBuf::new(buf.unfilled_mut());
				let l0 = read_buf.filled().len();
				match ready!(pin!(&mut self.inner)
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
							self.on_read(dl as u64);
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

	impl<R> Future for Recv<'_, R>
	where
		R: AsyncRead + Unpin,
	{
		type Output = Result<Vec<u8>, RecvPayloadError<io::Error>>;

		fn poll(
			mut self: Pin<&mut Self>,
			cx: &mut Context<'_>,
		) -> Poll<Self::Output> {
			self.advance(|r, buf| r.poll_read_tokio(cx, buf))
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
		) -> Result<T, RecvError<D::Error, io::Error>> {
			let mut payload =
				Recv::new(&mut self.pcb, &mut self.reader).await?;

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
		) -> Result<T, RecvError<D::Error, io::Error>> {
			self.receiver.recv().await
		}
	}
}

#[cfg(feature = "futures")]
#[cfg(not(feature = "tokio"))]
mod futures_impl {
	use super::*;

	use core::future::Future;
	use core::pin::{pin, Pin};
	use core::task::Context;

	use futures::AsyncRead;
	use std::io;

	impl<R> Reader<R>
	where
		R: AsyncRead + Unpin,
	{
		pub fn poll_read_futures(
			&mut self,
			cx: &mut Context,
			buf: &mut dyn BufMut,
		) -> Poll<io::Result<()>> {
			use io::ErrorKind as E;
			use Poll::*;

			while buf.has_remaining() {
				match ready!(pin!(&mut self.inner)
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
						self.on_read(n as u64);
					},
				}
			}

			Ready(Ok(()))
		}
	}

	impl<R> Future for Recv<'_, R>
	where
		R: AsyncRead + Unpin,
	{
		type Output = Result<Vec<u8>, RecvPayloadError<io::Error>>;

		fn poll(
			mut self: Pin<&mut Self>,
			cx: &mut Context<'_>,
		) -> Poll<Self::Output> {
			self.advance(|r, buf| r.poll_read_futures(cx, buf))
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
		) -> Result<T, RecvError<D::Error, io::Error>> {
			let mut payload =
				Recv::new(&mut self.pcb, &mut self.reader).await?;

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
		) -> Result<T, RecvError<D::Error, io::Error>> {
			self.receiver.recv().await
		}
	}
}
