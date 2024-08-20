use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Run {
	runs: u64,
}

/// Run _f_ repeatedly until `x` amount of time has passed.
pub fn run_for<F>(x: Duration, mut f: F) -> Run
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

		Run { runs }
	})
}

/// Run _f_ repeatedly for 30 seconds.
pub fn run_for_default_duration<F>(f: F) -> Run
where
	F: FnMut(),
{
	run_for(Duration::from_secs(30), f)
}
