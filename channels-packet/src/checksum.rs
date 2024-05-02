//! Calculate header checksums.

/// Checksum that implements the [Internet Checksum] algorithm.
///
/// [Internet Checksum]: https://en.wikipedia.org/wiki/Internet_checksum
#[derive(Debug, Clone)]
pub struct Checksum {
	state: u32,
}

impl Checksum {
	/// Create a new empty checksum.
	#[must_use]
	pub const fn new() -> Self {
		Self { state: 0 }
	}

	/// Update the checksum with `w`.
	pub fn update_u16(&mut self, w: u16) {
		self.state += u32::from(w);
	}

	/// Same as [`Checksum::update_u16`] but for use with the builder pattern.
	#[must_use]
	pub fn chain_update_u16(mut self, w: u16) -> Self {
		self.update_u16(w);
		self
	}

	/// Update the checksum from `buf`.
	#[allow(clippy::missing_panics_doc)]
	pub fn update(&mut self, data: &[u8]) {
		let mut iter = data.chunks_exact(2);

		iter.by_ref()
			.map(|x| -> [u8; 2] {
				x.try_into().expect(
					"chunks_exact() returned non N-sized chunk",
				)
			})
			.map(u16::from_be_bytes)
			.for_each(|w| self.update_u16(w));

		if let &[w] = iter.remainder() {
			self.update_u16(u16::from(w) << 8);
		}
	}

	/// Same as [`Checksum::update`] but for use with the builder pattern.
	#[must_use]
	pub fn chain_update(mut self, data: &[u8]) -> Self {
		self.update(data);
		self
	}

	/// Finalize the checksum.
	#[must_use]
	#[allow(clippy::cast_possible_truncation)]
	pub fn finalize(mut self) -> u16 {
		while (self.state >> 16) != 0 {
			self.state = (self.state >> 16) + (self.state & 0xffff);
		}

		!self.state as u16
	}
}

impl Default for Checksum {
	fn default() -> Self {
		Self::new()
	}
}

/// Calculate the checksum of `data`.
///
/// Equivalent to: `Checksum::new().chain_update(data).finalize()`.
#[must_use]
pub fn checksum(data: &[u8]) -> u16 {
	Checksum::new().chain_update(data).finalize()
}

/// Check whether the data is valid.
///
/// `data` must contain the checksum somewhere in it.
#[must_use]
pub fn verify(data: &[u8]) -> bool {
	checksum(data) == 0
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_checksum_impl() {
		fn test_case(data: &[u8], expected: u16) {
			let calculated = checksum(data);
			assert_eq!(
				expected, calculated,
				"{expected:#x?} != {calculated:#x?}"
			);
		}

		test_case(
			&[
				0x45, 0x00, 0x00, 0x97, 0x8b, 0x64, 0x40, 0x00, 0x40,
				0x06, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x94, 0x01, 0x01,
				0x01, 0x01,
			],
			0xa267,
		);

		test_case(
			&[
				0x45, 0x00, 0x02, 0x20, 0x54, 0x74, 0x40, 0x00, 0x37,
				0x06, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x0a, 0x00,
				0x00, 0x94,
			],
			0xe0ce,
		);

		test_case(
			&[
				0x45, 0x00, 0x00, 0xb3, 0x9b, 0xe9, 0x40, 0x00, 0xff,
				0x11, 0xf3, 0xc5, 0x0a, 0x00, 0x00, 0x8f, 0xe0, 0x00,
				0x00, 0xfb,
			],
			0x0000,
		);

		test_case(
			&[
				0x45, 0x00, 0x00, 0x73, 0x7e, 0x9b, 0x40, 0x00, 0x35,
				0x06, 0x4f, 0x1a, 0x03, 0x4a, 0x69, 0xf2, 0x0a, 0x00,
				0x00, 0x94,
			],
			0x0000,
		);
	}
}
