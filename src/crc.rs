static CRC: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_GSM);

pub fn checksum(bytes: &[u8]) -> u16 {
	CRC.checksum(bytes)
}
