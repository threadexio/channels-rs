use core::cmp;
use core::ops::{Deref, DerefMut};

use std::io;

use super::consts::*;
use super::header::HeaderRaw;
use super::types::*;

#[derive(Debug, Clone)]
pub struct Packet {
	inner: Vec<u8>,
	read_pos: PayloadLength,
	write_pos: PayloadLength,
}

impl Packet {
	/// Create a packet which can hold no data without growing.
	pub fn empty() -> Self {
		Self::with_capacity(Some(PayloadLength::MIN))
	}

	/// Create a packet which can hold `capacity` bytes of data without
	/// growing.
	///
	/// If `capacity` is `None`, then this function will allocate new
	/// a packet with the maximum size allowed.
	pub fn with_capacity(capacity: Option<PayloadLength>) -> Self {
		let capacity = match capacity {
			Some(l) => l.to_packet_length(),
			None => PacketLength::MAX,
		}
		.into();

		Self {
			inner: vec![0xcc; capacity],
			read_pos: PayloadLength::MIN,
			write_pos: PayloadLength::MIN,
		}
	}
}

#[allow(dead_code)]
impl Packet {
	/// Get the number of bytes this packet can hold.
	pub fn capacity(&self) -> PayloadLength {
		PayloadLength::try_from(self.payload().len()).unwrap()
	}

	/// Check whether the packet holds no data.
	pub fn is_empty(&self) -> bool {
		self.write_pos == PayloadLength::MIN
	}

	/// Check whether the packet has been filled with data completely.
	///
	/// When this method returns `true`, it does not mean the packet
	/// has reached its maximum capacity, but rather that the currently
	/// allocated buffer has been filled regardless of capacity.
	pub fn is_full(&self) -> bool {
		self.remaining_payload().is_empty()
	}

	/// Check whether the packet has reached its maximum capacity.
	///
	/// When this method returns `true`, the packet can no longer grow.
	pub fn has_reached_max_capacity(&self) -> bool {
		self.capacity() == PayloadLength::MAX
	}

	/// Grow the packet so it can fit at least `new_len` bytes.
	///
	/// This method guarantees that the packet is always able to fit
	/// at least `new_len` bytes in total. Note that this method might
	/// over-allocate if it sees fit, so `packet.capacity()` is not
	/// always going to be equal to `new_len`.
	///
	/// If `new_len` is less or equal to the capacity, the packet is
	/// neither grown nor truncated.
	///
	/// Generally the following holds true:
	/// ```ignore
	/// let old_capacity: usize = packet.capacity().into();
	///
	/// let new_len = PayloadLength::from(42);
	/// packet.grow_to(new_len);
	/// let new_len: usize = new_len.into();
	///
	/// let new_capacity: usize = packet.capacity().into();
	///
	/// assert!(old_capacity <= new_capacity);
	/// assert!(new_capacity >= new_len);
	/// ```
	pub fn grow_to(&mut self, new_len: PayloadLength) {
		if new_len <= self.capacity() {
			return;
		}

		let new_len: usize = new_len.into();

		let mut n: usize = self.capacity().into();
		if n == 0 {
			n = 1;
		}

		while n < new_len {
			n *= 2;
		}

		let n = cmp::min(n, MAX_PAYLOAD_SIZE);
		let n = PayloadLength::try_from(n).unwrap();
		let n = n.to_packet_length();
		self.inner.resize(n.into(), 0);
	}
}

#[allow(dead_code)]
impl Packet {
	/// Get the position of the write cursor.
	///
	/// The write cursor is always greater or equal to the read cursor
	/// and always less or equal to `packet.capacity()`.
	pub fn write_pos(&self) -> PayloadLength {
		self.write_pos
	}

	/// Get the position of the read cursor.
	///
	/// The read cursor is always less or equal to the write cursor.
	pub fn read_pos(&self) -> PayloadLength {
		self.read_pos
	}

	/// Set the position of the write cursor.
	///
	/// # Panics
	///
	/// If `pos` points outside the memory of this packet, i.e.
	/// `pos > packet.capacity()`.
	pub fn set_write_pos(&mut self, pos: PayloadLength) {
		assert!(
			pos <= self.capacity(),
			"write cursor must point inside the packet"
		);

		self.write_pos = pos;
	}

	/// Set the position of the read cursor.
	///
	/// # Panics
	///
	/// If `pos` points after the write cursor. i.e.
	pub fn set_read_pos(&mut self, pos: PayloadLength) {
		assert!(
			pos <= self.write_pos,
			"read cursor must point before the write cursor"
		);

		self.read_pos = pos;
	}

	/// Advance the write cursor by `n` bytes.
	///
	/// # Panics
	///
	/// If `n` causes the write cursor to go out of bounds.
	/// See: [`set_write_pos`].
	pub fn advance_write_by(&mut self, n: usize) {
		let write_pos: usize = self.write_pos.into();
		let new_pos = PayloadLength::try_from(write_pos + n).unwrap();

		self.set_write_pos(new_pos);
	}

	/// Advance the read cursor by `n` bytes.
	///
	/// # Panics
	///
	/// If `n` causes the write cursor to go out of bounds.
	/// See: [`set_read_pos`].
	pub fn advance_read_by(&mut self, n: usize) {
		let read_pos: usize = self.read_pos.into();
		let new_pos = PayloadLength::try_from(read_pos + n).unwrap();

		self.set_read_pos(new_pos);
	}

	/// Clear any data this packet holds.
	///
	/// This method just resets the the read and write cursors to their
	/// initial position.
	pub fn clear(&mut self) {
		self.set_read_pos(PayloadLength::MIN);
		self.set_write_pos(PayloadLength::MIN);
	}
}

#[allow(dead_code)]
impl Packet {
	#[inline]
	pub fn initialized(&self) -> &[u8] {
		let end = self.write_pos().to_packet_length().into();
		&self.inner[..end]
	}

	#[inline]
	pub fn header(&self) -> &HeaderRaw {
		(&self.inner[..HEADER_SIZE]).try_into().unwrap()
	}

	#[inline]
	pub fn header_mut(&mut self) -> &mut HeaderRaw {
		(&mut self.inner[..HEADER_SIZE]).try_into().unwrap()
	}

	#[inline]
	pub fn payload(&self) -> &[u8] {
		&self.inner[HEADER_SIZE..]
	}

	#[inline]
	pub fn payload_mut(&mut self) -> &mut [u8] {
		&mut self.inner[HEADER_SIZE..]
	}

	#[inline]
	pub fn filled_payload(&self) -> &[u8] {
		let r = ..self.write_pos.into();
		&self.payload()[r]
	}

	#[inline]
	pub fn filled_payload_mut(&mut self) -> &mut [u8] {
		let r = ..self.write_pos.into();
		&mut self.payload_mut()[r]
	}

	#[inline]
	pub fn remaining_payload(&self) -> &[u8] {
		let r = self.write_pos.into()..;
		&self.payload()[r]
	}

	#[inline]
	pub fn remaining_payload_mut(&mut self) -> &mut [u8] {
		let r = self.write_pos.into()..;
		&mut self.payload_mut()[r]
	}

	#[inline]
	pub fn read_payload(&self) -> &[u8] {
		let r = ..self.read_pos.into();
		&self.payload()[r]
	}

	#[inline]
	pub fn read_payload_mut(&mut self) -> &mut [u8] {
		let r = ..self.read_pos.into();
		&mut self.payload_mut()[r]
	}

	#[inline]
	pub fn unread_payload(&self) -> &[u8] {
		let r = self.read_pos.into()..;
		&self.payload()[r]
	}

	#[inline]
	pub fn unread_payload_mut(&mut self) -> &mut [u8] {
		let r = self.read_pos.into()..;
		&mut self.payload_mut()[r]
	}
}

impl io::Write for Packet {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		if !self.has_reached_max_capacity()
			&& buf.len() > self.remaining_payload().len()
		{
			let extra_bytes =
				buf.len() - self.remaining_payload().len();

			let capacity: usize = self.capacity().into();
			let new_len =
				cmp::min(capacity + extra_bytes, MAX_PAYLOAD_SIZE);

			let new_len = PayloadLength::try_from(new_len).unwrap();
			self.grow_to(new_len);
		}

		let n = copy_min_len(buf, self.remaining_payload_mut());
		self.advance_write_by(n);

		Ok(n)
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}

impl io::Read for Packet {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let n = copy_min_len(self.unread_payload(), buf);
		self.advance_read_by(n);
		Ok(n)
	}
}

fn copy_min_len(src: &[u8], dst: &mut [u8]) -> usize {
	let i = cmp::min(src.len(), dst.len());
	dst[..i].copy_from_slice(&src[..i]);
	i
}

#[derive(Debug, Clone)]
pub struct List {
	packets: Vec<Packet>,
}

impl List {
	/// Create a new empty list.
	pub fn empty() -> Self {
		Self::from_vec(Vec::new())
	}

	/// Create a new list with only one fully-allocated packet.
	pub fn new() -> Self {
		Self::from_vec(vec![Packet::with_capacity(None)])
	}

	/// Create a new list from a [`Vec`] of [`Packet`]s.
	pub fn from_vec(packets: Vec<Packet>) -> Self {
		Self { packets }
	}
}

impl List {
	/// Clear any data this list holds.
	///
	/// This method just calls [`Packet::clear`] on each packet in the
	/// list.
	pub fn clear(&mut self) {
		self.packets.iter_mut().for_each(|packet| packet.clear());
	}
}

impl Default for List {
	fn default() -> Self {
		Self::empty()
	}
}

impl io::Write for List {
	fn write(&mut self, mut buf: &[u8]) -> io::Result<usize> {
		if buf.is_empty() {
			return Ok(0);
		}

		let mut n = 0;
		let mut i = 0;
		loop {
			if self.packets.get(i).is_none() {
				let new_packet =
					match PayloadLength::try_from(buf.len()) {
						Ok(l) => Packet::with_capacity(Some(l)),
						Err(_) => Packet::with_capacity(None),
					};

				self.packets.push(new_packet);
			}
			let packet = &mut self.packets[i];

			let x = packet.write(buf)?;
			buf = &buf[x..];
			n += x;

			if buf.is_empty() {
				break;
			}

			i += 1;
		}

		Ok(n)
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}

impl io::Read for List {
	fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
		if buf.is_empty() {
			return Ok(0);
		}

		let mut n = 0;
		for packet in self.packets.iter_mut() {
			let x = packet.read(buf)?;
			buf = &mut buf[x..];
			n += x;

			if buf.is_empty() {
				break;
			}
		}

		Ok(n)
	}
}

impl Deref for List {
	type Target = Vec<Packet>;

	fn deref(&self) -> &Self::Target {
		&self.packets
	}
}

impl DerefMut for List {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.packets
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::io::{Read, Write};

	#[test]
	fn test_packet() {
		let mut p = Packet::empty();
		assert_eq!(p.capacity(), PayloadLength::from(0));

		let data: Vec<u8> = (0x00..0xff).collect();

		while !p.is_full() {
			let n = p.write(&data).unwrap();
			if n == 0 {
				break;
			}
		}

		let mut buf = vec![0u8; 0xff];

		while !p.unread_payload().is_empty() {
			let n = p.read(&mut buf).unwrap();
			if n == 0 {
				break;
			}

			assert!(&buf[..n].iter().enumerate().all(|(i, e)| {
				let i = i as u8;
				*e == i
			}));
		}
	}

	#[test]
	fn test_list() {
		let mut list = List::empty();

		let data: Vec<u8> = (0..0xff).collect();

		let mut n = 0;
		while n <= 2 * MAX_PACKET_SIZE {
			let x = list.write(&data).unwrap();
			assert_eq!(x, 255);

			n += x;
		}

		assert!(list.packets.len() >= 2);
	}
}
