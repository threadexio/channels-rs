use std::fmt;
use std::thread;
use std::time::{Duration, Instant};

use channels::Statistics;

/// Spawn a server and a client thread and wait for them to complete.
pub fn spawn_server_client<S, C, So, Co>(
	server: S,
	client: C,
) -> (So, Co)
where
	So: Send,
	Co: Send,
	S: FnOnce() -> So + Send,
	C: FnOnce() -> Co + Send,
{
	thread::scope(|scope| {
		let t1 = thread::Builder::new()
			.name("server".into())
			.spawn_scoped(scope, server)
			.unwrap();

		thread::sleep(Duration::from_secs(1));

		let t2 = thread::Builder::new()
			.name("client".into())
			.spawn_scoped(scope, client)
			.unwrap();

		(t1.join().unwrap(), t2.join().unwrap())
	})
}

/// Block until `f` returns `true`.
pub fn block_until<F>(mut f: F)
where
	F: FnMut() -> bool,
{
	loop {
		if f() {
			break;
		}

		thread::sleep(Duration::from_millis(500));
	}
}

/// Measure the execution time of _f_.
pub fn time<F, Output>(f: F) -> (Duration, Output)
where
	F: FnOnce() -> Output,
{
	let start = Instant::now();
	let output = f();
	let elapsed = start.elapsed();

	(elapsed, output)
}

pub struct Stats<'a> {
	pub duration: Duration,
	pub tx: &'a Statistics,
	pub rx: &'a Statistics,
}

impl fmt::Display for Stats<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let duration_s = self.duration.as_secs_f64();

		let packet_send_rate = self.tx.packets() as f64 / duration_s;
		let packet_recv_rate = self.rx.packets() as f64 / duration_s;

		let avg_tx_packet_size =
			self.tx.total_bytes() as f64 / self.tx.packets() as f64;
		let avg_rx_packet_size =
			self.rx.total_bytes() as f64 / self.rx.packets() as f64;

		let tx_rate_bps = self.tx.total_bytes() as f64 / duration_s;
		let tx_rate_kbps = tx_rate_bps / 1024.0;
		let tx_rate_mbps = tx_rate_kbps / 1024.0;

		let rx_rate_bps = self.rx.total_bytes() as f64 / duration_s;
		let rx_rate_kbps = rx_rate_bps / 1024.0;
		let rx_rate_mbps = rx_rate_kbps / 1024.0;

		#[rustfmt::skip]
		{
			writeln!(f, "finished in {:.5} seconds", duration_s)?;
			writeln!(f)?;
			writeln!(f, "total send operations:    {}", self.tx.ops())?;
			writeln!(f, "total receive operations: {}", self.rx.ops())?;
			writeln!(f)?;
			writeln!(f, "total packets sent:       {}", self.tx.packets())?;
			writeln!(f, "total packets received:   {}", self.rx.packets())?;
			writeln!(f)?;
			writeln!(f, "tx total: {} B", self.tx.total_bytes())?;
			writeln!(f, "rx total: {} B", self.rx.total_bytes())?;
			writeln!(f)?;
			writeln!(f, "packet send rate:         {:.3} packets/s", packet_send_rate)?;
			writeln!(f, "packet receive rate:      {:.3} packets/s", packet_recv_rate)?;
			writeln!(f)?;
			writeln!(f, "tx rate: {:.3} B/s = {:.3} kB/s = {:.3} MB/s", tx_rate_bps, tx_rate_kbps, tx_rate_mbps)?;
			writeln!(f, "rx rate: {:.3} B/s = {:.3} kB/s = {:.3} MB/s", rx_rate_bps, rx_rate_kbps, rx_rate_mbps)?;
			writeln!(f)?;
			writeln!(f, "average tx packet size:   {:.3} B", avg_tx_packet_size)?;
			writeln!(f, "average rx packet size:   {:.3} B", avg_rx_packet_size)?;
		};

		Ok(())
	}
}
