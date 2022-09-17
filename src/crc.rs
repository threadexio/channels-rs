//! Data validation with CRC.
//!
//! The `Sender` and `Receiver` types have a `Crc`
//! structure that holds the algorithm that will be used. If you need to change the
//! algorithm refer back to the [`crc`](https://crates.io/crates/crc) crate.
//!
//! # Example
//! ```rust
//! fn main() {
//! 	let (mut tx, mut rx) = ...;
//!
//! 	tx.crc.crc16.algorithm = &crc::CRC_16_ARC; // or any other
//! 	rx.crc.crc16.algorithm = &crc::CRC_16_ARC; // just make sure to change the other end too
//!
//! 	...
//! }
//! ```

/// Default CRC16 algorithm
pub const CRC16: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_GSM);

pub struct Crc {
	pub crc16: crc::Crc<u16>,
}

impl Default for Crc {
	fn default() -> Self {
		Self { crc16: CRC16 }
	}
}
