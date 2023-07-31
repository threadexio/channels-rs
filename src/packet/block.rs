use core::cmp;

use super::consts::*;
use super::header::*;

/// A buffer that holds a packet.
///
/// The read/write implementations operate on the payload of the packet.
///
/// A visualization of the buffer is:
///
/// ```not_rust
/// [  header  |             payload             ]
/// [  header  |       filled      |  remaining  ]
/// [  header  |  read  |  unread  |             ]
///                    /\         /\
///         payload_read_pos   payload_write_pos
/// ```
#[derive(Debug)]
pub struct Block {
	packet: Vec<u8>,
	payload_read_pos: usize,
	payload_write_pos: usize,
}

impl Block {
	pub fn new() -> Self {
		Self::with_payload_capacity(0)
	}

	pub fn with_payload_capacity(capacity: usize) -> Self {
		Self {
			packet: vec![0u8; HEADER_SIZE + capacity],
			payload_read_pos: 0,
			payload_write_pos: 0,
		}
	}

	/// Clear the payload from the buffer.
	pub fn clear(&mut self) {
		self.payload_read_pos = 0;
		self.payload_write_pos = 0;
	}

	/// Get the total number of bytes this packet can currently hold.
	///
	/// Includes the header.
	pub fn capacity(&self) -> usize {
		self.packet.len()
	}

	/// Get the length of the current payload inside the packet.
	pub fn payload_length(&self) -> PayloadLength {
		PayloadLength::from_usize(self.payload_write_pos).unwrap()
	}

	/// Get the length of the current packet.
	pub fn packet_length(&self) -> PacketLength {
		self.payload_length().to_packet_length()
	}

	/// Check whether the payload of this block is empty.
	pub fn is_payload_empty(&self) -> bool {
		self.payload_write_pos == 0
	}

	/// Check whether the payload of this block is full.
	pub fn is_payload_full(&self) -> bool {
		self.payload_write_pos == self.payload().len()
	}

	/// Grow the packet buffer by `extra` bytes.
	///
	/// Returns the new size of the packet.
	pub fn grow(&mut self, extra: usize) {
		if extra == 0 {
			return;
		}

		self.packet.resize(self.capacity() + extra, 0);
	}

	/// Get the entire packet up to the position of the payload cursor.
	pub fn packet(&self) -> &[u8] {
		let l = self.packet_length().as_usize();
		&self.packet[..l]
	}

	/// Advance the write head of this buffer by `n` bytes.
	///
	/// # Panics
	///
	/// This method will panic if `n` causes the write head to go out
	/// of bounds. `n` must be `<= block.remaining_payload().len()`.
	pub fn advance_write(&mut self, n: usize) {
		assert!(n <= self.remaining_payload().len());
		self.payload_write_pos += n;
	}

	/// Advance the read head of this buffer by `n` bytes.
	///
	/// # Panics
	///
	/// This method will panic if `n` causes the read head to go out
	/// of bounds. `n` must be `<= block.unread_payload().len()`.
	pub fn advance_read(&mut self, n: usize) {
		assert!(n <= self.unread_payload().len());
		self.payload_read_pos += n;
	}
}

impl Block {
	/// Get the slice corresponding to the header.
	pub fn header(&self) -> &[u8] {
		&self.packet[..Header::SIZE]
	}

	/// Get the mutable slice corresponding to the header.
	pub fn header_mut(&mut self) -> &mut [u8] {
		&mut self.packet[..Header::SIZE]
	}

	/// Get the slice corresponding to the payload.
	pub fn payload(&self) -> &[u8] {
		&self.packet[Header::SIZE..]
	}

	/// Get the mutable slice corresponding to the payload.
	pub fn payload_mut(&mut self) -> &mut [u8] {
		&mut self.packet[Header::SIZE..]
	}

	/// Get the slice corresponding to the filled payload.
	pub fn filled_payload(&self) -> &[u8] {
		&self.payload()[..self.payload_write_pos]
	}

	/// Get the mutable slice corresponding to the filled payload.
	pub fn filled_payload_mut(&mut self) -> &mut [u8] {
		let end = self.payload_write_pos;
		&mut self.payload_mut()[..end]
	}

	/// Get the slice corresponding to the remaining payload.
	pub fn remaining_payload(&self) -> &[u8] {
		&self.payload()[self.payload_write_pos..]
	}
	/// Get the mutable slice corresponding to the remaining payload.
	pub fn remaining_payload_mut(&mut self) -> &mut [u8] {
		let payload_end = self.payload_write_pos;
		&mut self.payload_mut()[payload_end..]
	}

	/// Get the slice corresponding to the read payload.
	pub fn read_payload(&self) -> &[u8] {
		&self.payload()[..self.payload_read_pos]
	}

	/// Get the mutable slice corresponding to the read payload.
	pub fn read_payload_mut(&mut self) -> &[u8] {
		let end = self.payload_read_pos;
		&mut self.payload_mut()[..end]
	}

	/// Get the slice corresponding to the filled payload.
	pub fn unread_payload(&self) -> &[u8] {
		&self.payload()[self.payload_read_pos..self.payload_write_pos]
	}

	/// Get the mutable slice corresponding to the filled payload.
	pub fn unread_payload_mut(&mut self) -> &mut [u8] {
		let start = self.payload_read_pos;
		let end = self.payload_write_pos;
		&mut self.payload_mut()[start..end]
	}
}

use std::io;

impl io::Write for Block {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		if buf.is_empty() {
			return Ok(0);
		}

		if self.remaining_payload().len() < buf.len() {
			let extra = buf.len() - self.remaining_payload().len();

			let new_size =
				cmp::min(self.capacity() + extra, MAX_PACKET_SIZE);

			let delta = new_size - self.capacity();
			self.grow(delta);
		}

		let n = copy_min_len(buf, self.remaining_payload_mut());
		self.advance_write(n);

		Ok(n)
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}

impl io::Read for Block {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		if buf.is_empty() {
			return Ok(0);
		}

		let n = copy_min_len(self.unread_payload(), buf);
		self.advance_read(n);

		Ok(n)
	}
}

fn copy_min_len(src: &[u8], dst: &mut [u8]) -> usize {
	let n = cmp::min(src.len(), dst.len());
	if n != 0 {
		dst[..n].copy_from_slice(&src[..n]);
	}
	n
}

#[test]
fn test_block() {
	use io::Write;

	let mut block = Block::new();

	let data: Vec<u8> = (1..=32).collect();

	let _ = dbg!(block.write(&data[..6]));
	let _ = dbg!(block.write(&[]));
	let _ = dbg!(block.write(&data[6..]));
}
