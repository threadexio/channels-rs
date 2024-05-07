use std::fmt;
use std::thread;
use std::time::{Duration, Instant};

use channels::Statistics;

pub mod units;

use units::{Bytes, Kilobytes, Megabytes};

#[derive(
	Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
pub struct Data {
	pub a: i32,
	pub b: usize,
	pub c: String,
}

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

pub struct TestResults<'a> {
	pub duration: Duration,
	pub stats: &'a Statistics,
}

impl fmt::Display for TestResults<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let duration_s = self.duration.as_secs_f64();

		let op_count = self.stats.ops();
		let packet_count = self.stats.packets();
		let total_bytes = self.stats.total_bytes();

		let op_rate = op_count as f64 / duration_s;
		let packet_rate = packet_count as f64 / duration_s;

		let io_rate_bps = total_bytes as f64 / duration_s;
		let io_rate_kbps = io_rate_bps / 1024.0;
		let io_rate_mbps = io_rate_kbps / 1024.0;

		let avg_packet_size =
			total_bytes as f64 / packet_count as f64;

		#[rustfmt::skip]
		{
			writeln!(f, "finished in {duration_s:.5}s")?;
			writeln!(f)?;
			writeln!(f, "total operations:    {op_count}")?;
			writeln!(f, "total packets:       {packet_count}")?;
			writeln!(f, "io total:            {} = {} = {}", Bytes(total_bytes), Kilobytes(total_bytes), Megabytes(total_bytes))?;
			writeln!(f)?;
			writeln!(f, "operation rate:      {op_rate:.3} operations/s")?;
			writeln!(f, "packet rate:         {packet_rate:.3} packets/s")?;
			writeln!(f, "io rate:             {io_rate_bps:.3} B/s = {io_rate_kbps:.3} kB/s = {io_rate_mbps:.3} MB/s")?;
			writeln!(f)?;
			writeln!(f, "average packet size: {avg_packet_size:.3} B")?;
		};

		Ok(())
	}
}
