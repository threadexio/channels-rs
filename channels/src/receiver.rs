//! Module containing the implementation for [`Receiver`].

use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::ready;
use core::task::{Context, Poll};

use alloc::vec::Vec;

use channels_io::prelude::*;
use channels_io::{BufMut, IoSlice};

use channels_packet::{Flags, Header};
use channels_serdes::Deserializer;

#[allow(unused_imports)]
use crate::common::{Pcb, Statistics};
use crate::error::RecvError;

/// The receiving-half of the channel. This is the same as [`std::sync::mpsc::Receiver`],
/// except for a [few key differences](crate).
///
/// See [crate-level documentation](crate).
#[derive(Debug, Clone)]
pub struct Receiver<T, R, D> {
	_marker: PhantomData<T>,
	reader: StatReader<R>,
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

impl<T, R, D> Receiver<T, R, D>
where
	R: AsyncRead + Unpin,
	D: Deserializer<T>,
{
	/// Attempts to receive a type `T` from the channel.
	///
	/// This function will return a future that will complete only when all the
	/// bytes of `T` have been received.
	pub fn recv(
		&mut self,
	) -> impl Future<Output = Result<T, RecvError<D::Error, R::Error>>> + '_
	{
		Recv::new(self)
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
	) -> Result<T, RecvError<D::Error, R::Error>> {
		Recv::new(self).poll_once(|r, buf| r.read_all(buf)).unwrap()
	}
}

/// An iterator over received messages.
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
	R: AsyncRead + Unpin,
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

	// TODO: Implement this in the appropriate trait when async iterators are
	//       stabilized. (https://github.com/rust-lang/rust/issues/79024)
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
	pub fn reader<R: Read>(
		self,
		reader: impl IntoReader<R>,
	) -> Builder<T, R, D> {
		Builder {
			_marker: PhantomData,
			reader: reader.into_reader(),
			deserializer: self.deserializer,
		}
	}

	/// Use this asynchronous reader.
	pub fn async_reader<R: AsyncRead>(
		self,
		reader: impl IntoAsyncReader<R>,
	) -> Builder<T, R, D> {
		Builder {
			_marker: PhantomData,
			reader: reader.into_async_reader(),
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
			reader: StatReader::new(self.reader),
			deserializer: self.deserializer,
			pcb: Pcb::new(),
			//state: State {
			//	payload: IoSlice::new(Vec::new()),
			//	state: StateEnum::INITIAL,
			//},
		}
	}
}

#[derive(Debug, Clone)]
struct StatReader<R> {
	inner: R,

	#[cfg(feature = "statistics")]
	statistics: Statistics,
}

impl<R> StatReader<R> {
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

impl<R> Read for StatReader<R>
where
	R: Read,
{
	type Error = R::Error;

	fn read_all(
		&mut self,
		mut buf: impl BufMut,
	) -> Poll<Result<(), Self::Error>> {
		let l0 = buf.remaining();
		let output = self.inner.read_all(&mut buf);
		let l1 = buf.remaining();

		let delta = l0 - l1;
		self.on_read(delta as u64);
		output
	}
}

impl<R> AsyncRead for StatReader<R>
where
	R: AsyncRead + Unpin,
{
	type Error = R::Error;

	fn poll_read_all(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		mut buf: impl BufMut,
	) -> Poll<Result<(), Self::Error>> {
		let l0 = buf.remaining();
		let output =
			Pin::new(&mut self.inner).poll_read_all(cx, &mut buf);
		let l1 = buf.remaining();

		let delta = l0 - l1;
		self.on_read(delta as u64);
		output
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

/// A future that will complete once a whole type `T` has been received.
#[must_use = "futures do nothing unless you `.await` them"]
struct Recv<'a, T, R, D> {
	recv: &'a mut Receiver<T, R, D>,
	payload: IoSlice<Vec<u8>>,
	state: State,
}

impl<'a, T, R, D> Recv<'a, T, R, D>
where
	D: Deserializer<T>,
{
	fn new(recv: &'a mut Receiver<T, R, D>) -> Self {
		Self {
			recv,
			state: State::INITIAL,
			payload: IoSlice::new(Vec::new()),
		}
	}

	fn poll_once<F, E>(
		&mut self,
		mut read_all: F,
	) -> Poll<Result<T, RecvError<D::Error, E>>>
	where
		F: FnMut(
			&mut StatReader<R>,
			&mut dyn BufMut,
		) -> Poll<Result<(), E>>,
	{
		use Poll::*;

		loop {
			match self.state {
				State::PartialHeader { ref mut part } => {
					match ready!(read_all(
						&mut self.recv.reader,
						part
					)) {
						Err(e) => {
							return Ready(Err(RecvError::Io(e)));
						},
						Ok(_) => {
							let header = Header::read_from(
								part.inner_ref(),
								&mut self.recv.pcb.id_gen,
							)
							.map_err(|e| {
								RecvError::Verify(e.into())
							})?;

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
				State::PartialPayload { ref header } => {
					match ready!(read_all(
						&mut self.recv.reader,
						&mut self.payload
					)) {
						Err(e) => {
							return Ready(Err(RecvError::Io(e)));
						},
						Ok(_) if header.flags & Flags::MORE_DATA => {
							self.state = State::INITIAL;
						},
						Ok(_) => {
							let res = self
								.recv
								.deserializer
								.deserialize(self.payload.inner_mut())
								.map_err(RecvError::Serde);

							return Ready(res);
						},
					}
				},
			}
		}
	}
}

impl<'a, T, R, D> Future for Recv<'a, T, R, D>
where
	R: AsyncRead + Unpin,
	D: Deserializer<T>,
{
	type Output = Result<T, RecvError<D::Error, R::Error>>;

	fn poll(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Self::Output> {
		self.get_mut()
			.poll_once(|r, buf| Pin::new(r).poll_read_all(cx, buf))
	}
}

/// Grow `vec` by `len` bytes and return the newly-allocated bytes as a slice.
fn reserve_slice_in_vec(vec: &mut Vec<u8>, len: usize) -> &mut [u8] {
	let start = vec.len();
	let new_len = usize::saturating_add(start, len);
	vec.resize(new_len, 0);
	&mut vec[start..new_len]
}
