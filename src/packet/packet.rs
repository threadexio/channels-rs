// TODO: remove me

use core::ops::{Deref, DerefMut};

use crate::error::VerifyError;
use crate::io::{BytesMut, BytesRef, Cursor};

use super::header::*;

pub struct Packet<T>(Cursor<T>);

impl<T> Packet<T> {
	pub const MAX_SIZE: usize = 0xffff;
	pub const MAX_PAYLOAD_SIZE: usize = Self::MAX_SIZE - Header::SIZE;
}

impl<T> Packet<T>
where
	T: BytesRef,
{
	/// Create a new packet from `inner`.
	///
	/// # Panics
	///
	/// If `inner.len() < Header::SIZE`.
	pub fn new(inner: T) -> Self {
		assert!(
			inner.as_ref().len() >= Header::SIZE,
			"channels packet buffer requires inner.len() >= {}",
			Header::SIZE
		);

		Self(Cursor::new(inner))
	}

	/// Clear the packet buffer of its contents.
	pub fn clear(&mut self) {
		self.0.set_position(0);
	}
}

impl<T> Packet<T>
where
	T: BytesRef,
{
	/// Get the entire buffer as a slice.
	///
	/// Other methods such as: [`header_slice`] or [`payload_slice`]
	/// should be preferred.
	pub fn as_slice(&self) -> &[u8] {
		self.0.as_slice()
	}

	/// Get a slice to the header.
	pub fn header_slice(&self) -> &[u8] {
		// SAFETY: The `new` constructor guarantees `self.0.len() >= HEADER_SIZE`
		&self.as_slice()[..Header::SIZE]
	}

	/// Get a cursor to the header.
	pub fn header(&self) -> Cursor<&[u8]> {
		Cursor::new(self.header_slice())
	}

	/// Get a slice to the payload.
	pub fn payload_slice(&self) -> &[u8] {
		// SAFETY: The `new` constructor guarantees `self.0.len() >= HEADER_SIZE`
		&self.as_slice()[Header::SIZE..]
	}

	/// Get a cursor to the payload.
	pub fn payload(&self) -> Cursor<&[u8]> {
		Cursor::new(self.payload_slice())
	}
}

impl<T> Packet<T>
where
	T: BytesMut,
{
	/// Get the entire buffer as a mutable slice.
	///
	/// Other methods such as: [`header_mut_slice`] or [`payload_mut_slice`]
	/// should be preferred.
	pub fn as_mut_slice(&mut self) -> &mut [u8] {
		self.0.as_mut_slice()
	}

	/// Get a mutable slice to the header.
	pub fn header_mut_slice(&mut self) -> &mut [u8] {
		// SAFETY: See `header_slice`.
		&mut self.as_mut_slice()[..Header::SIZE]
	}

	/// Get a cursor to the mutable header.
	pub fn header_mut(&mut self) -> Cursor<&mut [u8]> {
		Cursor::new(self.header_mut_slice())
	}

	/// Get a mutable slice to the payload.
	pub fn payload_mut_slice(&mut self) -> &mut [u8] {
		// SAFETY: See `payload_slice`.
		&mut self.as_mut_slice()[Header::SIZE..]
	}

	/// Get a cursor to the mutable payload.
	pub fn payload_mut(&mut self) -> Cursor<&mut [u8]> {
		Cursor::new(self.payload_mut_slice())
	}
}

impl<T> Packet<T>
where
	T: BytesRef,
{
	/// Read the header from the packet without verifying it.
	pub unsafe fn get_header_unchecked(&self) -> Header {
		Header::read_from_unchecked(self.header_slice())
	}

	/// Read the header from the packet and verify it.
	///
	/// **NOTE:** This method does not verify the `id` field.
	pub fn get_header(&self) -> Result<Header, VerifyError> {
		// SAFETY: `new()` guarantees length >= HEADER::SIZE
		Header::read_from(self.header_slice())
	}
}

impl<T> Packet<T>
where
	T: BytesMut,
{
	/// Prepare a packet ready to be sent.
	///
	/// No modification to the payload or the header slice after this
	/// point should occur.
	pub fn finalize(&mut self, header: &Header) {
		// SAFETY: `new()` guarantees length >= Header::SIZE
		header.write_to(self.header_mut_slice());
	}
}

impl<T> Deref for Packet<T> {
	type Target = Cursor<T>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> DerefMut for Packet<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
