use core::fmt;

const KILO: f64 = 10e+3;
const MEGA: f64 = 10e+6;

#[derive(Debug, Clone, Copy)]
pub struct Bytes(pub u64);

impl fmt::Display for Bytes {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} B", self.0)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Kilobytes(pub u64);

impl fmt::Display for Kilobytes {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:.2} kB", self.0 as f64 / KILO)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Megabytes(pub u64);

impl fmt::Display for Megabytes {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:.2} MB", self.0 as f64 / MEGA)
	}
}
