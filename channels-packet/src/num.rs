//! TODO: docs

macro_rules! unsigned_bit_num {
    (
        $(#[$attr:meta])*
        $vis:vis struct $T:ident ($repr:ty) {
            bits: $bits:literal,
        }
    ) => {
        $(#[$attr])*
        #[allow(non_camel_case_types)]
        $vis struct $T($repr);

        impl $T {
            /// TODO: docs
            pub const BITS: usize = $bits;
            /// TODO: docs
            pub const BIT_MASK: $repr = (1 << Self::BITS) - 1;

            /// TODO: docs
            pub const SIZE: usize = Self::BITS / 8;

            /// TODO: docs
            pub const MIN: Self = Self(0);
            /// TODO: docs
            pub const MAX: Self = Self(Self::BIT_MASK);

            /// TODO: docs
            /// # Safety
            ///
            /// The caller must ensure that `x` is valid.
            #[inline]
			#[must_use]
            pub const unsafe fn new_unchecked(x: $repr) -> Self {
                Self(x)
            }

            /// TODO: docs
            #[inline]
			#[must_use]
            pub const fn new(x: $repr) -> Option<Self> {
                if x & (!Self::BIT_MASK) == 0 {
                    Some(Self(x))
                } else{
                    None
                }
            }

            /// TODO: docs
            #[inline]
			#[must_use]
            pub const fn new_truncate(x: $repr) -> Self {
                Self(x & Self::BIT_MASK)
            }

            /// TODO: docs
            #[inline]
			#[must_use]
            pub const fn get(self) -> $repr {
                self.0
            }
        }
    };
}

unsigned_bit_num! {
	/// TODO: docs
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct u1(u8) {
		bits: 1,
	}
}

unsigned_bit_num! {
	/// TODO: docs
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct u2(u8) {
		bits: 2,
	}
}

unsigned_bit_num! {
	/// TODO: docs
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct u3(u8) {
		bits: 3,
	}
}

unsigned_bit_num! {
	/// TODO: docs
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct u4(u8) {
		bits: 4,
	}
}

unsigned_bit_num! {
	/// TODO: docs
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct u5(u8) {
		bits: 5,
	}
}

unsigned_bit_num! {
	/// TODO: docs
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct u6(u8) {
		bits: 6,
	}
}

unsigned_bit_num! {
	/// TODO: docs
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct u7(u8) {
		bits: 7,
	}
}

unsigned_bit_num! {
	/// TODO: docs
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct u24(u32) {
		bits: 24,
	}
}

unsigned_bit_num! {
	/// TODO: docs
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct u48(u64) {
		bits: 48,
	}
}
