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
/// ```no_run
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

/// Unwrap a `Poll<T>` to a `T`.
///
/// # Safety
///
/// The caller must ensure that `poll` is of the variant [`Poll::Ready`].
///
/// # Panics
///
/// Panics if `poll` is the [`Poll::Pending`] variant.
#[track_caller]
#[must_use]
pub fn unwrap_poll<T>(poll: Poll<T>) -> T {
	match poll {
		Poll::Ready(t) => t,
		Poll::Pending => {
			panic!("unwrap_poll tried to unwrap a pending value")
		},
	}
}
