use core::borrow::Borrow;
use core::marker::Unpin;

use ::tokio::io::{AsyncWrite, AsyncWriteExt};
use ::tokio::time::Duration;

use super::*;

impl<T, W, S> Sender<T, W, S>
where
	W: AsyncWrite + Unpin,
	S: Serializer<T>,
{
	/// Attempts to asynchronously send an object of type `T`
	/// through the writer.
	///
	/// For the blocking version of this method, see [`Sender::send_blocking`].
	///
	/// # Example
	/// ```no_run
	/// use channels::Sender;
	///
	/// #[tokio::main]
	/// async fn main() {
	///     let writer = tokio::io::sink();
	///     let mut sender = Sender::<i32, _, _>::new(writer);
	///
	///     sender.send(42_i32).await.unwrap();
	/// }
	/// ```
	pub async fn send<D>(
		&mut self,
		data: D,
	) -> Result<(), SendError<S::Error>>
	where
		D: Borrow<T>,
	{
		let data = data.borrow();
		self.packets.clear();

		self.serialize_t_to_packets(data)?;
		let packets =
			finalize_packets(&mut self.packets, &mut self.pcb);
		for packet in packets {
			self.writer.write_all(packet.initialized()).await?;
		}

		#[cfg(feature = "statistics")]
		self.writer.stats.update_sent_time();

		Ok(())
	}

	/// Attempts to asynchronously send an object of type `T` through
	/// the reader with a timeout.
	///
	/// If the object could not be sent in the duration specified by
	/// `timeout`, all data is cleared and this method returns [`SendError::Timeout`].
	///
	/// # Example
	/// ```no_run
	/// use channels::Sender;
	/// use std::time::Duration;
	///
	/// #[tokio::main]
	/// async fn main() {
	///     let writer = tokio::io::sink();
	///     let mut sender = Sender::<i32, _, _>::new(writer);
	///
	///     sender.send_timeout(42_i32, Duration::from_secs(1)).await.unwrap();
	/// }
	/// ```
	pub async fn send_timeout<D>(
		&mut self,
		data: D,
		timeout: Duration,
	) -> Result<(), SendError<S::Error>>
	where
		D: Borrow<T>,
	{
		let fut = async { self.send(data).await };
		let r = ::tokio::time::timeout(timeout, fut).await;

		match r {
			Ok(v) => v,
			Err(_) => Err(SendError::Timeout),
		}
	}

	/// Attempts to asynchronously send an object of type `T` through
	/// the underlying asynchronous writer.
	///
	/// It is not to be confused with [`Sender::send_blocking`]. This
	/// method is only available for asynchronous writers and its only
	/// purpose is to serve as a bridge between asynchronous and
	/// synchronous code. That said, it is almost always preferable to
	/// use directly the async API and `.await` where necessary.
	///
	/// You can call this method from inside an asynchronous runtime,
	/// but please note that it **will** block the entire runtime. In
	/// other words, any other tasks will not run until this completes.
	/// For this reason, it is not advices to use this in an synchronous
	/// context.
	///
	/// # Example
	/// ```no_run
	/// use channels::Sender;
	///
	/// fn main() {
	///     let writer = tokio::io::sink();
	///     let mut sender = Sender::<i32, _, _>::new(writer);
	///
	///     sender.blocking_send(42_i32).unwrap();
	/// }
	/// ```
	pub fn blocking_send<D>(
		&mut self,
		data: D,
	) -> Result<(), SendError<S::Error>>
	where
		D: Borrow<T>,
	{
		crate::util::block_on(async { self.send(data).await })?
	}
}
