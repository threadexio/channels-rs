#![allow(unused_macros)]

macro_rules! newtype {
    (
		$(#[$attr:meta])*
		$name:ident
	) => {
		$(#[$attr])*
		#[derive(Debug)]
		pub struct $name<T>(pub T);
	};
}

macro_rules! impl_newtype_read {
    ($name:ident: $($bounds:tt)*) => {
			impl<T> $crate::Reader for $name<T>
			where
				T: $($bounds)*
			{
				type Inner = T;

				fn get(&self) -> &Self::Inner {
					&self.0
				}

				fn get_mut(&mut self) -> &mut Self::Inner {
					&mut self.0
				}

				fn into_inner(self) -> Self::Inner {
					self.0
				}
			}

			impl<T> $crate::IntoReader<$name<T>> for T
			where
				T: $($bounds)*
			{
				fn into_reader(self) -> $name<T> {
					$name(self)
				}
			}
		};
	}

macro_rules! impl_newtype_write {
	($name:ident: $($bounds:tt)*) => {
		impl<T> $crate::Writer for $name<T>
		where
			T: $($bounds)*
		{
			type Inner = T;

			fn get(&self) -> &Self::Inner {
				&self.0
			}

			fn get_mut(&mut self) -> &mut Self::Inner {
				&mut self.0
			}

			fn into_inner(self) -> Self::Inner {
				self.0
			}
		}

		impl<T> $crate::IntoWriter<$name<T>> for T
		where
			T: $($bounds)*
		{
			fn into_writer(self) -> $name<T> {
				$name(self)
			}
		}
	}
}

use impl_newtype_read;
use impl_newtype_write;
use newtype;

#[allow(unused_imports)]
mod prelude {
	pub(super) use super::{
		super::read::*, super::write::*, impl_newtype_read,
		impl_newtype_write, newtype,
	};
	pub(super) use crate::{Contiguous, ContiguousMut};
}

macro_rules! declare_impls {
	($mod:ident :: $name:ident, $($tail:tt)*) => {
		mod $mod;
		pub use self::$mod::$name;

		declare_impls! { $($tail)* }
	};
	($mod:ident :: $name:ident if ($($cfg:tt)+), $($tail:tt)*) => {
		#[cfg($($cfg)*)]
		mod $mod;
		#[cfg($($cfg)*)]
		pub use self::$mod::$name;

		declare_impls! { $($tail)* }
	};
	() => {};
}

declare_impls! {
	native::Native,
	std::Std         if (feature = "std"),
	tokio::Tokio     if (feature = "tokio"),
	futures::Futures if (feature = "futures"),
}
