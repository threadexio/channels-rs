/*
 * Automatically generated from `tools/header.py`. Do not edit!
 *
 * Header spec: spec/header.json
 */

pub unsafe fn unsafe_get_version(buf: &[u8]) -> u16 {
	let x = read_offset(buf, 0);
	u16::from_be(x)
}

pub unsafe fn unsafe_set_version(buf: &mut [u8], value: u16) {
	write_offset(buf, 0, u16::to_be(value));
}

pub unsafe fn unsafe_get_packet_length(buf: &[u8]) -> u16 {
	let x = read_offset(buf, 2);
	u16::from_be(x)
}

pub unsafe fn unsafe_set_packet_length(buf: &mut [u8], value: u16) {
	write_offset(buf, 2, u16::to_be(value));
}

pub unsafe fn unsafe_get_header_checksum(buf: &[u8]) -> u16 {
	let x = read_offset(buf, 4);
	x
}

pub unsafe fn unsafe_set_header_checksum(buf: &mut [u8], value: u16) {
	write_offset(buf, 4, value);
}

pub unsafe fn unsafe_get_flags(buf: &[u8]) -> u8 {
	let x = read_offset(buf, 6);
	x
}

pub unsafe fn unsafe_set_flags(buf: &mut [u8], value: u8) {
	write_offset(buf, 6, value);
}

pub unsafe fn unsafe_get_packet_id(buf: &[u8]) -> u8 {
	let x = read_offset(buf, 7);
	x
}

pub unsafe fn unsafe_set_packet_id(buf: &mut [u8], value: u8) {
	write_offset(buf, 7, value);
}

pub const HEADER_HASH: u16 = 0xfd3f;

pub const HEADER_SIZE: usize = 8;
