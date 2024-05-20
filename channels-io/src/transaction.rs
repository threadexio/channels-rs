#![allow(missing_docs)] // TODO: docs

use core::pin::Pin;
use core::task::{Context, Poll};

use alloc::vec::Vec;

use crate::{AsyncWrite, Write};

#[derive(Debug)]
pub struct WriteTransaction<'a, W: ?Sized> {
	writer: &'a mut W,
	buf: Option<&'a mut Vec<u8>>,
	wants_flush: bool,
}

impl<'a, W: ?Sized> WriteTransaction<'a, W> {
	pub fn buffered(writer: &'a mut W, buf: &'a mut Vec<u8>) -> Self {
		Self { writer, buf: Some(buf), wants_flush: false }
	}

	pub fn unbuffered(writer: &'a mut W) -> Self {
		Self { writer, buf: None, wants_flush: false }
	}

	#[inline]
	#[must_use]
	pub fn writer(&self) -> &W {
		self.writer
	}

	#[inline]
	#[must_use]
	pub fn writer_mut(&mut self) -> &mut W {
		self.writer
	}
}

impl<'a, W: Write + ?Sized> Write for WriteTransaction<'a, W> {
	type Error = W::Error;

	fn write_slice(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		match self.buf.as_mut() {
			None => self.writer.write_slice(buf),
			Some(x) => {
				x.extend_from_slice(buf);
				Ok(buf.len())
			},
		}
	}

	fn flush_once(&mut self) -> Result<(), Self::Error> {
		match self.buf {
			None => self.writer.flush(),
			Some(_) => {
				self.wants_flush = true;
				Ok(())
			},
		}
	}
}

impl<'a, W: Write + ?Sized> WriteTransaction<'a, W> {
	pub fn finish_sync(self) -> Result<(), W::Error> {
		let Self { buf, wants_flush, writer } = self;

		match buf {
			None => Ok(()),
			Some(buf) => writer.write(buf),
		}?;

		if wants_flush {
			writer.flush()?;
		}

		Ok(())
	}
}

impl<'a, W: AsyncWrite + ?Sized> AsyncWrite
	for WriteTransaction<'a, W>
{
	type Error = W::Error;

	fn poll_write_slice(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &[u8],
	) -> Poll<Result<usize, Self::Error>> {
		match self.buf.as_mut() {
			None => {
				Pin::new(&mut self.writer).poll_write_slice(cx, buf)
			},
			Some(x) => {
				x.extend_from_slice(buf);
				Poll::Ready(Ok(buf.len()))
			},
		}
	}

	fn poll_flush_once(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
	) -> Poll<Result<(), Self::Error>> {
		match self.buf {
			None => Pin::new(&mut self.writer).poll_flush_once(cx),
			Some(_) => {
				self.wants_flush = true;
				Poll::Ready(Ok(()))
			},
		}
	}
}

impl<'a, W: AsyncWrite + ?Sized> WriteTransaction<'a, W> {
	pub async fn finish_async(self) -> Result<(), W::Error> {
		let Self { buf, wants_flush, writer } = self;

		match buf {
			None => Ok(()),
			Some(buf) => writer.write(buf).await,
		}?;

		if wants_flush {
			writer.flush().await?;
		}

		Ok(())
	}
}
