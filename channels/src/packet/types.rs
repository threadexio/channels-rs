use super::consts::*;
use crate::util::flags;

macro_rules! impl_num {
	(
		$(#[$attr:meta])*
		$name:ident {
			type: $t:ty,
			min: $min:expr,
			max: $max:expr,
			from: [ $($from_t:ty),* ],
			try_from: [ $($try_from_t:ty),* ],
			into: [ $(
				$into_t:ty $(: $into_t_alt_fn:ident)?
			),* ],
		}
	) => {
		$(#[$attr])*
		#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct $name($t);

		impl $name {
			pub const MIN: Self = Self($min);
			pub const MAX: Self = Self($max);

			$(
				$(
					pub fn $into_t_alt_fn(self) -> $into_t {
						self.into()
					}
				)?
			)*
		}

		$(
			impl From<$from_t> for $name {
				fn from(value: $from_t) -> Self {
					Self(value as $t)
				}
			}
		)*

		$(
			impl TryFrom<$try_from_t> for $name {
				type Error = ();

				fn try_from(value: $try_from_t) -> Result<Self, Self::Error> {
					let valid = (Self::MIN.0 as $try_from_t)..=(Self::MAX.0 as $try_from_t);
					if valid.contains(&value) {
						Ok(Self(value as $t))
					} else {
						Err(())
					}
				}
			}
		)*

		$(
			impl From<$name> for $into_t {
				fn from(value: $name) -> Self {
					value.0 as $into_t
				}
			}
		)*
	};
}

impl_num! {
	/// The length of the payload inside one packet.
	///
	/// The following holds true for this type:
	///
	/// - `0 <= l <= MAX_PAYLOAD_SIZE`
	PayloadLength {
		type: u16,
		min: 0,
		max: MAX_PAYLOAD_SIZE as u16,
		from: [ u8 ],
		try_from: [ u16, u32, u64, usize ],
		into: [
			u16: as_u16,
			u32: as_u32,
			u64: as_u64,
			usize: as_usize
		],
	}
}

impl_num! {
	/// The length of one packet.
	///
	/// The following holds true for this type:
	///
	/// - `HEADER_SIZE <= l <= MAX_PACKET_SIZE`
	PacketLength {
		type: u16,
		min: HEADER_SIZE as u16,
		max: MAX_PACKET_SIZE as u16,
		from: [ ],
		try_from: [ u16, u32, u64, usize ],
		into: [
			u16: as_u16,
			u32: as_u32,
			u64: as_u64,
			usize: as_usize
		],
	}
}

impl PayloadLength {
	pub fn to_packet_length(self) -> PacketLength {
		PacketLength(self.0 + PacketLength::MIN.0)
	}
}

impl PacketLength {
	pub fn to_payload_length(self) -> PayloadLength {
		PayloadLength(self.0 - Self::MIN.0)
	}
}

flags! {
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
	pub struct Flags(u8) {
		const MORE_DATA = 0b_1000_0000;
	}
}

#[derive(
	Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Id(pub u8);

impl Id {
	pub fn next(&mut self) -> Self {
		let old = *self;
		self.0 = self.0.wrapping_add(1);
		old
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Checksum(pub u16);

impl Checksum {
	pub fn calculate(buf: &[u8]) -> Self {
		let checksum = unsafe {
			let mut addr = buf.as_ptr().cast::<u16>();
			let mut left = buf.len();
			let mut sum: u32 = 0;

			while left >= 2 {
				sum += u32::from(*addr);
				addr = addr.add(1);
				left -= 2;
			}

			if left == 1 {
				let addr = addr.cast::<u8>();
				sum += u32::from(*addr);
			}

			loop {
				let upper = sum >> 16;
				if upper == 0 {
					break;
				}

				sum = upper + (sum & 0xffff);
			}

			!(sum as u16)
		};

		Self(checksum)
	}

	pub fn verify(buf: &[u8]) -> bool {
		Self::calculate(buf) == Self(0)
	}
}
