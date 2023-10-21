#![allow(unused)]

use core::task::Poll;

/// Types which can be used as a byte slice.
pub trait Bytes: AsRef<[u8]> {
	/// Convert this type to a byte slice.
	fn as_bytes(&self) -> &[u8] {
		self.as_ref()
	}
}

/// Types which can be used as a mutable byte slice.
pub trait BytesMut: Bytes + AsMut<[u8]> {
	/// Convert this type to a mutable byte slice.
	fn as_mut_bytes(&mut self) -> &mut [u8] {
		self.as_mut()
	}
}

impl<T: AsRef<[u8]>> Bytes for T {}
impl<T: Bytes + AsMut<[u8]>> BytesMut for T {}

/// A macro to automatically create the [newtype pattern] for generic types.
///
/// # Syntax
///
/// `newtype! { [type name] for: [trait bound] + ... }`
///
/// It creates a new tuple struct named `type name` which is generic over its
/// one field. That generic is bounded by `trait bound`. [`Deref`] and [`DerefMut`]
/// are also implemented giving access to the inner generic type.
///
/// # Example
///
/// ```ignore
/// use core::fmt::{Debug, Display};
///
/// newtype! { MyType for: Debug + Display }
///
/// let a = MyType("42");
/// println!("{a:?} {a}");
/// ```
///
/// [`Deref`]: core::ops::Deref
/// [`DerefMut`]: core::ops::DerefMut
/// [newtype pattern]: https://doc.rust-lang.org/rust-by-example/generics/new_types.html
macro_rules! newtype {
	($newtype:ident for: $($bounds:tt)+) => {
		pub struct $newtype<T>(T)
		where
			T: $($bounds)+;

		impl<T> ::core::ops::Deref for $newtype<T>
		where
			T: $($bounds)+
		{
			type Target = T;

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}

		impl<T> ::core::ops::DerefMut for $newtype<T>
		where
			T: $($bounds)+
		{
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.0
			}
		}
	}
}
pub(crate) use newtype;

/// Extension trait for [`Poll`].
pub trait PollExt<T>: Sized {
	/// Returns the contained [`Poll::Ready`] value consuming the `self` value.
	///
	/// # Panics
	///
	/// Panics if the value is a [`Poll::Pending`] with a panic message provided
	/// by `msg`.
	#[track_caller]
	fn expect(self, msg: &str) -> T;

	/// Returns the contained [`Poll::Ready`] value consuming the `self` value.
	///
	/// # Panics
	///
	/// Panics if the value is a [`Poll::Pending`].
	#[track_caller]
	fn unwrap(self) -> T {
		self.expect("unwrap called on a `Poll::Pending`")
	}

	/// Returns the contained [`Poll::Ready`] value or `other` if the value was
	/// [`Poll::Pending`].
	#[track_caller]
	fn unwrap_or(self, other: T) -> T {
		self.unwrap_or_else(|| other)
	}

	/// Returns the contained [`Poll::Ready`] value or computes if from _f_.
	#[track_caller]
	fn unwrap_or_else<F>(self, f: F) -> T
	where
		F: FnOnce() -> T;

	/// Returns the contained [`Poll::Ready`] value or the default value of `T`
	/// if the value was [`Poll::Pending`].
	#[track_caller]
	fn unwrap_or_default(self) -> T
	where
		T: Default,
	{
		self.unwrap_or_else(|| T::default())
	}
}

impl<T> PollExt<T> for Poll<T> {
	#[track_caller]
	fn expect(self, msg: &str) -> T {
		#[cold]
		#[inline(never)]
		#[track_caller]
		fn panic_pending(msg: &str) -> ! {
			panic!("{}", msg)
		}

		match self {
			Poll::Pending => panic_pending(msg),
			Poll::Ready(v) => v,
		}
	}

	#[track_caller]
	fn unwrap_or_else<F>(self, f: F) -> T
	where
		F: FnOnce() -> T,
	{
		match self {
			Poll::Pending => f(),
			Poll::Ready(v) => v,
		}
	}
}

/// Copy the largest sub-slice of `src` possible into `dst`.
///
/// Returns the number of individual elements copied.
pub fn copy_slice<T: Copy>(src: &[T], dst: &mut [T]) -> usize {
	match core::cmp::min(src.len(), dst.len()) {
		0 => 0,
		n => {
			dst[..n].copy_from_slice(&src[..n]);
			n
		},
	}
}
