use core::cmp;

use super::consts::*;
use super::header::HeaderRaw;
use super::types::*;

/// A buffer that holds a packet.
///
/// The read/write implementations operate on the payload of the packet.
///
/// A visualization of the buffer is:
///
/// ```not_rust
/// [                  capacity                  ]
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
	/// Create a new block with `0` payload capacity.
	pub fn new() -> Self {
		Self::with_payload_capacity(PayloadLength::from(0))
	}

	/// Create a new block with `capacity` payload capacity.
	pub fn with_payload_capacity(capacity: PayloadLength) -> Self {
		Self::with_packet_capacity(capacity.to_packet_length())
	}

	/// Create a new block with `capacity` packet capacity.
	pub fn with_packet_capacity(capacity: PacketLength) -> Self {
		Self {
			packet: vec![0u8; capacity.into()],
			payload_read_pos: 0,
			payload_write_pos: 0,
		}
	}
}

impl Block {
	/// Get the total number of bytes this block can currently hold.
	pub fn packet_capacity(&self) -> PacketLength {
		// SAFETY: `self.packet` is always smaller than `MAX_PACKET_SIZE`.
		PacketLength::try_from(self.packet.len()).unwrap()
	}

	/// Get the total number of bytes this block's payload buffer can
	/// currently hold.
	pub fn payload_capacity(&self) -> PayloadLength {
		PayloadLength::try_from(self.payload().len()).unwrap()
	}

	/// Grow the payload buffer to fit at least `new_capacity` bytes.
	pub fn grow_payload_to(&mut self, new_capacity: PayloadLength) {
		if new_capacity <= self.payload_capacity() {
			return;
		}

		let mut actual_new_capacity = self.payload_capacity().into();
		if actual_new_capacity == 0 {
			actual_new_capacity = 1;
		}

		let new_capacity = new_capacity.into();
		while actual_new_capacity < new_capacity {
			actual_new_capacity *= 2;

			if actual_new_capacity > MAX_PAYLOAD_SIZE {
				actual_new_capacity = MAX_PAYLOAD_SIZE;
				break;
			}
		}

		// SAFETY: actual_new_capacity <= MAX_PAYLOAD_SIZE
		let actual_new_capacity =
			PayloadLength::try_from(actual_new_capacity).unwrap();

		self.grow_payload_to_exact(actual_new_capacity);
	}

	/// Grow the packet to fit `new_capacity` bytes.
	pub fn grow_packet_to(&mut self, new_capacity: PacketLength) {
		self.grow_payload_to(new_capacity.to_payload_length())
	}

	/// Grow the payload buffer to fit exactly `new_capacity` bytes.
	fn grow_payload_to_exact(&mut self, new_capacity: PayloadLength) {
		self.grow_packet_to_exact(new_capacity.to_packet_length())
	}

	/// Grow the packet to fit exactly `new_capacity` bytes.
	fn grow_packet_to_exact(&mut self, new_capacity: PacketLength) {
		// SAFETY: `self.packet` will never exceed `MAX_PACKET_SIZE`
		//         because: `new_capacity <= MAX_PACKET_SIZE`.
		//
		//         See: `PacketLength`
		self.packet.resize(new_capacity.into(), 0);
	}
}

impl Block {
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

	/// Clear the payload buffer.
	///
	/// This method just resets the cursor positions to `0`.
	pub fn clear_payload(&mut self) {
		self.payload_read_pos = 0;
		self.payload_write_pos = 0;
	}
}

impl Block {
	/// Get the length of the payload inside the buffer.
	pub fn payload_length(&self) -> PayloadLength {
		// SAFETY: `self.payload_write_pos` is always guaranteed to be
		//         less than `MAX_PAYLOAD_SIZE`.
		PayloadLength::try_from(self.payload_write_pos).unwrap()
	}

	/// Get the length of the current packet.
	///
	/// Equivalent to: `block.payload_length().to_packet_length()`.
	pub fn packet_length(&self) -> PacketLength {
		self.payload_length().to_packet_length()
	}

	/// Get a slice to the entire packet in the buffer.
	///
	/// Only include the `header` and `filled` sections.
	///
	/// See: [`Block`]
	pub fn packet(&self) -> &[u8] {
		let l = self.packet_length().into();
		&self.packet[..l]
	}
}

impl Block {
	/// Get the slice corresponding to the header.
	pub fn header(&self) -> &HeaderRaw {
		// SAFETY: HeaderRaw is always of length `HEADER_SIZE`.
		(&self.packet[..HEADER_SIZE]).try_into().unwrap()
	}

	/// Get the mutable slice corresponding to the header.
	pub fn header_mut(&mut self) -> &mut HeaderRaw {
		// SAFETY: HeaderRaw is always of length `HEADER_SIZE`.
		(&mut self.packet[..HEADER_SIZE]).try_into().unwrap()
	}

	/// Get the slice corresponding to the payload.
	///
	/// The returned slice's length is `<= MAX_PAYLOAD_SIZE`.
	pub fn payload(&self) -> &[u8] {
		&self.packet[HEADER_SIZE..]
	}

	/// Get the mutable slice corresponding to the payload.
	///
	/// See: [`payload`]
	pub fn payload_mut(&mut self) -> &mut [u8] {
		&mut self.packet[HEADER_SIZE..]
	}

	/// Get the slice corresponding to the filled payload.
	///
	/// The returned slice's length is `<= MAX_PAYLOAD_SIZE`
	pub fn filled_payload(&self) -> &[u8] {
		&self.payload()[..self.payload_write_pos]
	}

	/// Get the mutable slice corresponding to the filled payload.
	///
	/// See: [`filled_payload`]
	pub fn filled_payload_mut(&mut self) -> &mut [u8] {
		let end = self.payload_write_pos;
		&mut self.payload_mut()[..end]
	}

	/// Get the slice corresponding to the remaining payload.
	///
	/// The returned slice's length is `<= MAX_PAYLOAD_SIZE`
	pub fn remaining_payload(&self) -> &[u8] {
		&self.payload()[self.payload_write_pos..]
	}
	/// Get the mutable slice corresponding to the remaining payload.
	///
	/// See: [`remaining_payload`]
	pub fn remaining_payload_mut(&mut self) -> &mut [u8] {
		let payload_end = self.payload_write_pos;
		&mut self.payload_mut()[payload_end..]
	}

	/// Get the slice corresponding to the read payload.
	///
	/// The returned slice's length is `<= MAX_PAYLOAD_SIZE`
	pub fn read_payload(&self) -> &[u8] {
		&self.payload()[..self.payload_read_pos]
	}

	/// Get the mutable slice corresponding to the read payload.
	///
	/// See: [`read_payload`]
	pub fn read_payload_mut(&mut self) -> &[u8] {
		let end = self.payload_read_pos;
		&mut self.payload_mut()[..end]
	}

	/// Get the slice corresponding to the filled payload.
	///
	/// The returned slice's length is `<= MAX_PAYLOAD_SIZE`
	pub fn unread_payload(&self) -> &[u8] {
		&self.payload()[self.payload_read_pos..self.payload_write_pos]
	}

	/// Get the mutable slice corresponding to the filled payload.
	///
	/// See: [`unread_payload`]
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

		// if the available space in the payload buffer is less than
		// the length of the buffer we want to write, then calculate
		// their difference and allocate

		let new_capacity = cmp::min(
			self.filled_payload().len() + buf.len(),
			MAX_PAYLOAD_SIZE,
		);
		let new_capacity =
			PayloadLength::try_from(new_capacity).unwrap();

		self.grow_payload_to(new_capacity);

		//self.ensure_payload_capacity(
		//	self.filled_payload().len() + buf.len(),
		//);

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
