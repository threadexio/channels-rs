/// The frame header checksum.
#[derive(Debug, Clone)]
pub struct Checksum {
	state: u32,
}

impl Checksum {
	/// Create a new empty [`Checksum`].
	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		Self { state: 0 }
	}

	/// Update the checksum from `x`.
	///
	/// # Panics
	///
	/// If `x` does not have even length.
	pub fn update(&mut self, x: &[u8]) -> &mut Self {
		assert!(x.len() & 0x1 == 0, "length of slice must be even");

		x.chunks_exact(2).for_each(|word| {
			let word = unsafe {
				word.as_ptr().cast::<u16>().read_unaligned()
			};
			self.state += u32::from(word);
		});

		self
	}

	/// Finalize the checksum and produce a final value.
	#[inline]
	#[must_use]
	pub fn finalize(&self) -> u16 {
		!fold_u32_to_u16(self.state)
	}

	/// Calculate the checksum of `data`.
	///
	/// This is a shorthand for: `Checksum::new().update(data).finalize()`.
	#[inline]
	#[must_use]
	pub fn checksum(data: &[u8]) -> u16 {
		Self::new().update(data).finalize()
	}
}

impl Default for Checksum {
	fn default() -> Self {
		Self::new()
	}
}

#[allow(clippy::cast_possible_truncation)]
#[inline]
const fn fold_u32_to_u16(mut x: u32) -> u16 {
	while x >> 16 != 0 {
		x = (x >> 16) + (x & 0xffff);
	}

	x as u16
}

#[cfg(test)]
mod tests {
	use super::*;

	macro_rules! tests {
        ($(
            $test_name:ident {
                input: $input:expr,
                expected: $expected:expr,
            },
        )*) => {
            $(
                #[test]
                fn $test_name() {
                    let input: &'static [u8] = $input;
                    let expected: [u8; 2] = $expected;

                    assert_eq!(
                        Checksum::checksum(input),
                        u16::from_ne_bytes(expected),
                        concat!("`", stringify!($test_name), "` failed")
                    );
                }
            )*
        };
    }

	tests! {
		test_checksum_ip_packet_calculate {
			input: &[0x45, 0x00, 0x00, 0x73, 0x00, 0x00, 0x40, 0x00, 0x40, 0x11, 0x00, 0x00, 0xc0, 0xa8, 0x00, 0x01, 0xc0, 0xa8, 0x00, 0xc7],
			expected: [0xb8, 0x61],
		},
		test_checksum_ip_packet_verify {
			input: &[0x45, 0x00, 0x00, 0x73, 0x00, 0x00, 0x40, 0x00, 0x40, 0x11, 0xb8, 0x61, 0xc0, 0xa8, 0x00, 0x01, 0xc0, 0xa8, 0x00, 0xc7],
			expected: [0x00, 0x00],
		},

		test_checksum_random_1_calculate {
			input: &[0x1c, 0x3b, 0xe6, 0x6f, 0xc4, 0xdc, 0xd5, 0x70, 0x30, 0x3f, 0xca, 0xb5, 0x72, 0x8d, 0x00, 0x00],
			expected: [0xf5, 0x84],
		},
		test_checksum_random_1_verify {
			input: &[0x1c, 0x3b, 0xe6, 0x6f, 0xc4, 0xdc, 0xd5, 0x70, 0x30, 0x3f, 0xca, 0xb5, 0x72, 0x8d, 0xf5, 0x84],
			expected: [0x00, 0x00],
		},

		test_checksum_random_2_calculate {
			input: &[0xae, 0x3e, 0x0d, 0x98, 0xbd, 0x16, 0xa2, 0xef, 0xac, 0x70, 0x9f, 0x49, 0x5e, 0xf3, 0x00, 0x00],
			expected: [0x39, 0x75],
		},
		test_checksum_random_2_verify {
			input: &[0xae, 0x3e, 0x0d, 0x98, 0xbd, 0x16, 0xa2, 0xef, 0xac, 0x70, 0x9f, 0x49, 0x5e, 0xf3, 0x39, 0x75],
			expected: [0x00, 0x00],
		},

		test_checksum_random_3_calculate {
			input: &[0x4d, 0xa7, 0x01, 0x69, 0xe3, 0x6b, 0xfb, 0xf5, 0xf2, 0x2f, 0x08, 0x61, 0x47, 0x0a, 0x00, 0x00],
			expected: [0x8f, 0xf2],
		},
		test_checksum_random_3_verify {
			input: &[0x4d, 0xa7, 0x01, 0x69, 0xe3, 0x6b, 0xfb, 0xf5, 0xf2, 0x2f, 0x08, 0x61, 0x47, 0x0a, 0x8f, 0xf2],
			expected: [0x00, 0x00],
		},
	}

	#[test]
	#[should_panic = "length of slice must be even"]
	fn test_calculate_checksum_invalid_slice() {
		let _ = Checksum::checksum(&[0x00, 0x00, 0x00]);
	}
}
