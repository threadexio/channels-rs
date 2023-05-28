use core::ops;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PacketId(u8);

impl PacketId {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn next_id(&mut self) -> &mut Self {
		self.0 = self.0.wrapping_add(1);
		self
	}
}

impl From<u8> for PacketId {
	fn from(value: u8) -> Self {
		Self(value)
	}
}

impl From<PacketId> for u8 {
	fn from(value: PacketId) -> Self {
		value.0
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PacketFlags(u8);

impl PacketFlags {
	pub const MORE_DATA: Self = Self(0b_1000_0000);

	pub fn zero() -> Self {
		Self(0)
	}

	pub unsafe fn new_unchecked(f: u8) -> Self {
		Self(f)
	}
}

impl From<PacketFlags> for u8 {
	fn from(value: PacketFlags) -> Self {
		value.0
	}
}

impl ops::BitAnd for PacketFlags {
	type Output = bool;

	fn bitand(self, rhs: Self) -> Self::Output {
		self.0 & rhs.0 != 0
	}
}

impl ops::BitOr for PacketFlags {
	type Output = Self;

	fn bitor(self, rhs: Self) -> Self::Output {
		Self(self.0 | rhs.0)
	}
}

impl ops::BitOrAssign for PacketFlags {
	fn bitor_assign(&mut self, rhs: Self) {
		self.0 |= rhs.0;
	}
}

impl ops::BitXor for PacketFlags {
	type Output = Self;

	fn bitxor(self, rhs: Self) -> Self::Output {
		Self(self.0 & !rhs.0)
	}
}

impl ops::BitXorAssign for PacketFlags {
	fn bitxor_assign(&mut self, rhs: Self) {
		self.0 &= !rhs.0;
	}
}
