use core::ops;

use crate::error::*;
use crate::io::OwnedBuf;
use crate::util::{read_offset, write_offset};

macro_rules! packet_fields {
	(@field
		$field_name:ident : {
			type: $field_ty:ty,
			offset: $field_byte_offset:expr,
			get: {
				fn: $field_getter_fn_ident:ident,
				$(vis: $field_getter_fn_vis:vis,)?
			},
			set: {
				fn: $field_setter_fn_ident:ident,
				$(vis: $field_setter_fn_vis:vis,)?
			},
			$(ser_fn: $field_ser_fn:expr,)?
			$(de_fn: $field_de_fn:expr,)?
		},
		$($tail:tt)*
	) => {
		$($field_getter_fn_vis)? fn $field_getter_fn_ident(&self) -> $field_ty {
			let x = read_offset::<$field_ty>(self.inner.as_slice(), $field_byte_offset);
			$(
				let x = $field_de_fn(x);
			)?
			x
		}

		$($field_setter_fn_vis)? fn $field_setter_fn_ident(&mut self, $field_name: $field_ty) {
			let x = $field_name;
			$(
				let x = $field_ser_fn(x);
			)?
			write_offset(self.inner.as_mut_slice(), $field_byte_offset, x);
		}

		packet_fields!( @field $($tail)* );
	};
	(@field) => {};
	($packet_struct:ident {
		$($tail:tt)*
	}) => {
		impl $packet_struct {
			packet_fields!( @field $($tail)* );
		}
	};
}

pub struct PacketBuf {
	inner: OwnedBuf,
}

impl PacketBuf {
	pub const HEADER_SIZE: usize = 8;
}

packet_fields! {
	PacketBuf {
		version: {
			type: u16,
			offset: 0,
			get: { fn: get_version, },
			set: { fn: set_version, },
			ser_fn: u16::to_be,
			de_fn: u16::from_be,
		},
		length: {
			type: u16,
			offset: 2,
			get: { fn: get_length, vis: pub, },
			set: { fn: set_length, vis: pub, },
			ser_fn: u16::to_be,
			de_fn: u16::from_be,
		},
		header_checksum: {
			type: u16,
			offset: 4,
			get: { fn: get_header_checksum, },
			set: { fn: set_header_checksum, },
			ser_fn: u16::to_be,
			de_fn: u16::from_be,
		},
		id: {
			type: u16,
			offset: 6,
			get: { fn: get_id, vis: pub, },
			set: { fn: set_id, vis: pub, },
			ser_fn: u16::to_be,
			de_fn: u16::from_be,
		},
	}
}

impl PacketBuf {
	pub const MAX_PACKET_SIZE: usize = 0xffff;

	pub fn new() -> Self {
		Self { inner: OwnedBuf::new(Self::MAX_PACKET_SIZE) }
	}
}

impl PacketBuf {
	pub fn header(&self) -> &[u8] {
		&self.inner.as_slice()[..Self::HEADER_SIZE]
	}

	pub fn header_mut(&mut self) -> &mut [u8] {
		&mut self.inner.as_mut_slice()[..Self::HEADER_SIZE]
	}

	pub fn payload(&self) -> &[u8] {
		&self.inner.as_slice()[Self::HEADER_SIZE..]
	}

	pub fn payload_mut(&mut self) -> &mut [u8] {
		&mut self.inner.as_mut_slice()[Self::HEADER_SIZE..]
	}
}

impl PacketBuf {
	const VERSION: u16 = 0x2;

	fn calculate_header_checksum(&mut self) -> u16 {
		self.set_header_checksum(0);
		crate::crc::checksum(self.header())
	}

	fn update_header_checksum(&mut self) {
		let c = self.calculate_header_checksum();
		self.set_header_checksum(c);
	}

	pub fn finalize(&mut self) {
		self.set_version(Self::VERSION);
		self.update_header_checksum();
	}

	pub fn verify_header(&mut self) -> Result<()> {
		if self.get_version() != Self::VERSION {
			return Err(Error::VersionMismatch);
		}

		{
			let unverified = self.get_header_checksum();
			let calculated = self.calculate_header_checksum();

			if unverified != calculated {
				return Err(Error::ChecksumError);
			}
		}

		if (self.get_length() as usize) < Self::HEADER_SIZE {
			return Err(Error::SizeLimit);
		}

		Ok(())
	}
}

impl ops::Deref for PacketBuf {
	type Target = OwnedBuf;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl ops::DerefMut for PacketBuf {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn correct_endianness() {
		let mut packet = PacketBuf::new();

		packet.set_version(1);
		assert_eq!(packet.get_version(), 1);

		packet.set_length(3);
		assert_eq!(packet.get_length(), 3);

		packet.set_header_checksum(4);
		assert_eq!(packet.get_header_checksum(), 4);

		assert_eq!(packet.header(), &[0, 1, 0, 3, 0, 4, 0, 0]);
	}
}
