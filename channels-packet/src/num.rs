use crate::{consts::HEADER_SIZE_U16, util::static_assert};

macro_rules! impl_bounded_num_type {
	(
		Self = $Self:path,
		Repr = $Repr:ty,
		Max = $max:expr,
		Min = $min:expr,
	) => {
		/// Minimum value of this type.
		pub const MIN: Self = Self($min as $Repr);

		/// Maximum value of this type.
		pub const MAX: Self = Self($max as $Repr);

		/// # Safety
		///
		/// The caller must ensure `x` is contained in the range: `MIN..=MAX`.
		#[inline]
		#[track_caller]
		#[must_use]
		pub const unsafe fn new_unchecked(x: $Repr) -> Self {
			assert!(Self::MIN.0 <= x && x <= Self::MAX.0);
			Self(x)
		}

		/// Returns `None` only if `x` is not contained in the range: `MIN..=MAX`.
		#[inline]
		#[must_use]
		pub const fn new(x: $Repr) -> Option<Self> {
			if (Self::MIN.0 <= x) && (x <= Self::MAX.0) {
				Some(unsafe { Self::new_unchecked(x) })
			} else {
				None
			}
		}

		/// Returns the `MAX` if `x` is greater than `MAX` and `MIN` if `x` is
		/// less than `MIN`.
		#[inline]
		#[must_use]
		pub const fn new_saturating(mut x: $Repr) -> Self {
			if x < Self::MIN.0 {
				x = Self::MIN.0;
			} else if x > Self::MAX.0 {
				x = Self::MAX.0;
			}

			unsafe { Self::new_unchecked(x) }
		}
	};
}

static_assert!(
	u16::MAX as u64 <= usize::MAX as u64,
	"cannot build on platforms where usize is smaller than u16"
);

/// A numeric type that represents any valid packet length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PacketLength(u16);

impl PacketLength {
	impl_bounded_num_type! {
		Self = PacketLength,
		Repr = u16,
		Max = u16::MAX,
		Min = HEADER_SIZE_U16,
	}

	/// Get the length as a [`usize`].
	#[inline]
	#[must_use]
	pub const fn as_usize(self) -> usize {
		// SAFETY: usize is always greater or equal to u16
		self.0 as usize
	}

	/// Get the length as a [`u16`].
	#[inline]
	#[must_use]
	pub const fn as_u16(self) -> u16 {
		self.0
	}

	/// Convert this packet length to a payload length.
	#[inline]
	#[must_use]
	pub const fn to_payload_length(self) -> PayloadLength {
		unsafe {
			// SAFETY: PacketLength::MIN <= self <= PacketLength::MAX
			//     <=> HEADER_SIZE_U16 <= self <= u16::MAX
			//     <=> 0 <= self - HEADER_SIZE_U16 <= u16::MAX - HEADER_SIZE_U16
			//     <=> PayloadLength::MIN <= self - HEADER_SIZE_U16 <= PayloadLength::MAX
			static_assert!(PayloadLength::MIN.0 == 0);
			static_assert!(
				PayloadLength::MAX.0 == u16::MAX - HEADER_SIZE_U16
			);

			PayloadLength::new_unchecked(self.0 - HEADER_SIZE_U16)
		}
	}
}

/// A numeric type that represents any valid payload length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PayloadLength(u16);

impl PayloadLength {
	impl_bounded_num_type! {
		Self = PayloadLength,
		Repr = u16,
		Max = PacketLength::MAX.as_u16() - HEADER_SIZE_U16,
		Min = 0,
	}

	/// Get the length as a [`usize`].
	#[inline]
	#[must_use]
	pub const fn as_usize(self) -> usize {
		// SAFETY: usize is always greater or equal to u16
		self.0 as usize
	}

	/// Get the length as a [`u16`].
	#[inline]
	#[must_use]
	pub const fn as_u16(self) -> u16 {
		self.0
	}

	/// Convert this payload length to a packet length.
	#[inline]
	#[must_use]
	pub const fn to_packet_length(self) -> PacketLength {
		unsafe {
			// SAFETY: PayloadLength::MIN <= self <= PayloadLength::MAX
			//     <=> 0 <= self <= PacketLength::MAX - HEADER_SIZE_U16
			//     <=> HEADER_SIZE_U16 <= self + HEADER_SIZE_U16 <= PacketLength::MAX
			//     <=> PacketLength::MIN <= self + HEADER_SIZE_U16 <= PacketLength::MAX
			static_assert!(PacketLength::MIN.0 == HEADER_SIZE_U16);
			static_assert!(PacketLength::MAX.0 == u16::MAX);

			PacketLength::new_unchecked(self.0 + HEADER_SIZE_U16)
		}
	}
}

/// A `u16` in big-endian byte order.
///
/// This type is `repr(transparent)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
#[allow(non_camel_case_types)]
pub struct u16be(u16);

impl From<u16> for u16be {
	fn from(value: u16) -> Self {
		Self(value.to_be())
	}
}

impl From<u16be> for u16 {
	fn from(value: u16be) -> Self {
		u16::from_be(value.0)
	}
}
