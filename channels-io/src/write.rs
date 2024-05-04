/// This trait allows writing bytes to a writer.
///
/// Types implementing this trait are called "writers".
///
/// Writers have only one method, [`Write::write()`] which will attempt to write
/// some bytes to the sink.
pub trait Write {
	/// Error type for [`Write::write()`].
	type Error;

	/// Write `buf` to the writer.
	///
	/// Upon return, this function must guarantee that either: a) `buf` has been
	/// fully written, `buf.has_remaining()` should return `false`. b)
	/// an error has occurred that _cannot_ be handled immediately.
	///
	/// If `buf` has been written to the writer, then this function must return
	/// with [`Ok(())`](Ok). In any other case it must return an [`Err`].
	fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error>;

	/// Flush this writer ensuring all bytes reach their destination.
	///
	/// Upon return, this function must ensure that all bytes written to the
	/// writer by previous calls to [`Write::write()`] have reached their
	/// destination.
	fn flush(&mut self) -> Result<(), Self::Error>;
}

macro_rules! forward_impl_write {
	($to:ty) => {
		type Error = <$to>::Error;

		fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
			<$to>::write(self, buf)
		}

		fn flush(&mut self) -> Result<(), Self::Error> {
			<$to>::flush(self)
		}
	};
}

impl<T: Write + ?Sized> Write for &mut T {
	forward_impl_write!(T);
}

#[cfg(feature = "alloc")]
impl<T: Write + ?Sized> Write for alloc::boxed::Box<T> {
	forward_impl_write!(T);
}
