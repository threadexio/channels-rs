//! Structures that hold statistic information about channels.
//!
//! See: [`statistics`] feature.
//!
//! [`statistics`]: crate#features

use std::time::{Duration, Instant};

/// Statistic information about for [`Sender`](crate::Sender).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendStats {
	total_sent: usize,
	last_sent: Instant,
	delta_sent: Duration,
}

/// Statistic information about for [`Receiver`](crate::Receiver).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecvStats {
	total_received: usize,
	last_received: Instant,
	delta_received: Duration,
}

impl SendStats {
	pub(crate) fn new() -> Self {
		Self {
			total_sent: 0,
			last_sent: Instant::now(),
			delta_sent: Duration::from_secs(0),
		}
	}

	pub(crate) fn add_sent(&mut self, amt: usize) {
		self.total_sent += amt;
	}

	pub(crate) fn update_sent_time(&mut self) {
		let now = Instant::now();
		self.delta_sent = now - self.last_sent;
		self.last_sent = now;
	}

	/// Returns the total amount of bytes sent by this [`Sender`](crate::Sender).
	pub fn total_sent(&self) -> usize {
		self.total_sent
	}

	/// Returns the timestamp when the last packet was sent.
	pub fn last_sent(&self) -> Instant {
		self.last_sent
	}

	/// Returns the duration between the last 2 packets.
	pub fn delta_sent(&self) -> Duration {
		self.delta_sent
	}
}

impl RecvStats {
	pub(crate) fn new() -> Self {
		Self {
			total_received: 0,
			last_received: Instant::now(),
			delta_received: Duration::from_secs(0),
		}
	}

	pub(crate) fn add_received(&mut self, amt: usize) {
		self.total_received += amt;
	}

	pub(crate) fn update_received_time(&mut self) {
		let now = Instant::now();
		self.delta_received = now - self.last_received;
		self.last_received = now;
	}

	/// Returns the total amount of bytes received by this [`Receiver`](crate::Receiver).
	pub fn total_received(&self) -> usize {
		self.total_received
	}

	/// Returns the timestamp when the last whole packet was received.
	pub fn last_received(&self) -> Instant {
		self.last_received
	}

	/// Returns the duration between the last 2 packets.
	pub fn delta_received(&self) -> Duration {
		self.delta_received
	}
}
