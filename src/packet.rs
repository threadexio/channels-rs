#[inline]
fn read_offset<T>(buf: &[u8], offset: usize) -> T {
	unsafe { buf.as_ptr().add(offset).cast::<T>().read() }
}

#[inline]
fn write_offset<T>(buf: &mut [u8], offset: usize, value: T) {
	unsafe {
		buf.as_mut_ptr().add(offset).cast::<T>().write(value);
	}
}

macro_rules! packet {
	(
		$(#[$pkt_attr:meta])*
		$pkt_name:ident {$(
		$offset:literal @ $name:ident : $typ:ty {
			ser = ($ser_fn:expr);
			de = ($de_fn:expr);
			$(get = ($getter_vis:vis);)?
			$(set = ($setter_vis:vis);)?
		},
	)*}) => {
		#[derive(Debug)]
		$(#[$pkt_attr])*
		pub struct $pkt_name {
			inner: Box<[u8]>
		}

		#[allow(dead_code)]
		impl $pkt_name {
			pub const SIZE: usize = 0 $(+ ::std::mem::size_of::<$typ>())*;

			pub fn new() -> Self {
				Self {
					inner: vec![0u8; Self::SIZE].into_boxed_slice()
				}
			}

			#[inline]
			pub fn raw(&self) -> &[u8] {
				&self.inner
			}

			#[inline]
			pub fn raw_mut(&mut self) -> &mut [u8] {
				&mut self.inner
			}

			$(
				concat_idents::concat_idents!(getter = get_, $name {
					$($getter_vis)? fn getter(&self) -> $typ {
						$de_fn(read_offset::<$typ>(&self.inner[..], $offset))
					}
				});

				concat_idents::concat_idents!(setter = set_, $name {
					$($setter_vis)? fn setter(&mut self, $name: $typ) -> &mut Self {
						write_offset::<$typ>(&mut self.inner[..], $offset, $ser_fn($name));
						self
					}
				});
			)*
		}
	};
}

packet! {
	Header {
		0 @ version:  u16 { ser = (u16::to_be); de = (u16::from_be);                           }, // protocol version
		2 @ id:       u16 { ser = (u16::to_be); de = (u16::from_be); get = (pub); set = (pub); }, // packet id
		4 @ length:   u16 { ser = (u16::to_be); de = (u16::from_be); get = (pub); set = (pub); }, // data length
		6 @ checksum: u16 { ser = (u16::to_be); de = (u16::from_be);                           }, // header checksum
	}
}

pub const MAX_PACKET_SIZE: usize = u16::MAX as usize;
pub const PROTOCOL_VERSION: u16 = 1;

impl Header {
	pub fn finalize(&mut self) -> &[u8] {
		self.set_version(PROTOCOL_VERSION);
		self.set_checksum(0);

		let checksum = crate::crc::checksum(self.raw());
		self.set_checksum(checksum);

		self.raw()
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
		let mut header =
			Self { inner: Vec::from(bytes).into_boxed_slice() };

		if header.get_version() != PROTOCOL_VERSION {
			return Err(Error::VersionMismatch);
		}

		let unverified = header.get_checksum();
		header.set_checksum(0);
		let calculated = crate::crc::checksum(header.raw());

		if unverified != calculated {
			return Err(Error::ChecksumError);
		}

		Ok(header)
	}
}

use bincode::Options;
macro_rules! bincode {
	() => {
		bincode::options()
			.reject_trailing_bytes()
			.with_big_endian()
			.with_fixint_encoding()
			.with_no_limit()
	};
}

use crate::error::*;

#[inline]
pub fn serialize<T: serde::Serialize>(data: &T) -> Result<Vec<u8>> {
	Ok(bincode!().serialize(data)?)
}

#[inline]
pub fn deserialize<T: serde::de::DeserializeOwned>(
	data: &[u8],
) -> Result<T> {
	Ok(bincode!().deserialize::<T>(data)?)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_read_offset() {
		let buffer: &[u8] = &[0, 0, 0, 1];

		assert_eq!(read_offset::<u32>(buffer, 0), u32::to_be(1));

		let buffer: &[u8] = &[42, 23, 0, 0, 0, 1, 42, 42];

		assert_eq!(read_offset::<u32>(buffer, 2), u32::to_be(1));
	}

	#[test]
	fn test_write_offset() {
		let buffer: &mut [u8] = &mut [0, 0, 0, 0];

		write_offset::<u32>(buffer, 0, u32::to_be(1));
		assert_eq!(buffer, &[0, 0, 0, 1]);

		let buffer: &mut [u8] =
			&mut [42, 23, 23, 212, 233, 35, 42, 42];

		write_offset::<u32>(buffer, 2, u32::to_be(1));
		assert_eq!(buffer, &[42, 23, 0, 0, 0, 1, 42, 42]);
	}

	#[test]
	fn test_header() {
		let mut header = Header::new();

		assert_eq!(header.get_version(), 0);
		header.set_version(42);
		assert_eq!(header.raw()[..2], u16::to_be_bytes(42));
		assert_eq!(header.get_version(), 42);
	}
}
