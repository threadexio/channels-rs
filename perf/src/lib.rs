use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// Run _f_ repeatedly until `x` amount of time has passed.
pub fn run_for<F>(x: Duration, mut f: F) -> Report
where
	F: FnMut(),
{
	let exit = AtomicBool::new(false);
	thread::scope(|scope| {
		scope.spawn(|| {
			thread::sleep(x);
			exit.store(true, Ordering::Relaxed);
		});

		let mut runs = 0;
		while !exit.load(Ordering::Relaxed) {
			f();
			runs += 1;
		}

		Report { runs, dur: x }
	})
}

/// Run _f_ repeatedly for 30 seconds.
pub fn run_for_default_duration<F>(f: F) -> Report
where
	F: FnMut(),
{
	run_for(Duration::from_secs(10), f)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Report {
	dur: Duration,
	runs: u64,
}

impl fmt::Display for Report {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let dur = self.dur.as_secs_f64();
		let runs = self.runs;

		let avg_dur_per_run = dur / (runs as f64);

		writeln!(
			f,
			r#"took {dur} for {runs} runs (avg {avg_dur_per_run} / run)"#,
			dur = NormalizedTime(dur),
			avg_dur_per_run = NormalizedTime(avg_dur_per_run)
		)
	}
}

static TIME_UNITS_SUFFIXES: &[&str] = &["s", "ms", "μs", "ns", "ps"];

struct NormalizedTime(f64);

impl fmt::Display for NormalizedTime {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut x = self.0;

		for unit in TIME_UNITS_SUFFIXES {
			if x < 1.0 {
				x *= 1000.0;
			} else {
				write!(f, "{x:.3}{unit}")?;
				break;
			}
		}

		Ok(())
	}
}
