pub const CRC32: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);

pub fn checksum32(data: &[u8]) -> u32 {
	CRC32.checksum(data)
}
