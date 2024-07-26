use core::pin::Pin;
use core::task::{Context, Poll};

use alloc::vec::Vec;

use pin_project::pin_project;

use crate::transaction::{
	AsyncWriteTransaction, Buffered, Unbuffered, WriteTransaction,
};
use crate::{AsyncWrite, Write};

/// Kind of the transaction.
#[derive(Debug)]
pub enum WriteTransactionKind<'a> {
	/// A [`Buffered`] transaction.
	Buffered(&'a mut Vec<u8>),
	/// An [`Unbuffered`] transaction.
	Unbuffered,
}

/// A unified interface for calling code to choose which transaction kind it wants
/// at runtime.
#[derive(Debug)]
#[pin_project(project = WriteTransactionVariantProj)]
pub enum WriteTransactionVariant<'a, W> {
	/// A [`Buffered`] transaction.
	Buffered(#[pin] Buffered<'a, W>),
	/// An [`Unbuffered`] transaction.
	Unbuffered(#[pin] Unbuffered<W>),
}

impl<'a, W> WriteTransactionVariant<'a, W> {
	/// Create a [`Buffered`] transaction.
	pub fn buffered(writer: W, buf: &'a mut Vec<u8>) -> Self {
		Self::Buffered(Buffered::new(writer, buf))
	}

	/// Create an [`Unbuffered`] transaction.
	pub fn unbuffered(writer: W) -> Self {
		Self::Unbuffered(Unbuffered::new(writer))
	}

	/// Create a new transaction based on `kind`.
	pub fn new(writer: W, kind: WriteTransactionKind<'a>) -> Self {
		match kind {
			WriteTransactionKind::Buffered(buf) => {
				Self::buffered(writer, buf)
			},
			WriteTransactionKind::Unbuffered => {
				Self::unbuffered(writer)
			},
		}
	}

	/// Get a reference to the underlying writer.
	pub fn writer(&self) -> &W {
		match self {
			Self::Buffered(x) => x.writer(),
			Self::Unbuffered(x) => x.writer(),
		}
	}

	/// Get a mutable reference to the underlying writer.
	pub fn writer_mut(&mut self) -> &mut W {
		match self {
			Self::Buffered(x) => x.writer_mut(),
			Self::Unbuffered(x) => x.writer_mut(),
		}
	}
}

impl<'a, W> Write for WriteTransactionVariant<'a, W>
where
	W: Write,
{
	type Error = W::Error;

	fn write_slice(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		match self {
			Self::Buffered(x) => x.write_slice(buf),
			Self::Unbuffered(x) => x.write_slice(buf),
		}
	}

	fn flush_once(&mut self) -> Result<(), Self::Error> {
		match self {
			Self::Buffered(x) => x.flush_once(),
			Self::Unbuffered(x) => x.flush_once(),
		}
	}
}

impl<'a, W> WriteTransaction for WriteTransactionVariant<'a, W>
where
	W: Write,
{
	fn finish(self) -> Result<(), Self::Error> {
		match self {
			Self::Buffered(x) => x.finish(),
			Self::Unbuffered(x) => x.finish(),
		}
	}
}

impl<'a, W> AsyncWrite for WriteTransactionVariant<'a, W>
where
	W: AsyncWrite,
{
	type Error = W::Error;

	fn poll_write_slice(
		self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, Self::Error>> {
		match self.project() {
			WriteTransactionVariantProj::Buffered(x) => {
				x.poll_write_slice(cx, buf)
			},
			WriteTransactionVariantProj::Unbuffered(x) => {
				x.poll_write_slice(cx, buf)
			},
		}
	}

	fn poll_flush_once(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		match self.project() {
			WriteTransactionVariantProj::Buffered(x) => {
				x.poll_flush_once(cx)
			},
			WriteTransactionVariantProj::Unbuffered(x) => {
				x.poll_flush_once(cx)
			},
		}
	}
}

impl<'a, W> AsyncWriteTransaction for WriteTransactionVariant<'a, W>
where
	W: AsyncWrite,
{
	fn poll_finish(
		self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		match self.project() {
			WriteTransactionVariantProj::Buffered(x) => {
				x.poll_finish(cx)
			},
			WriteTransactionVariantProj::Unbuffered(x) => {
				x.poll_finish(cx)
			},
		}
	}
}
