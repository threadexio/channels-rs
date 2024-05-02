#![allow(unused_macros)]

macro_rules! newtype {
    (
		$(#[$attr:meta])*
		$name:ident
	) => {
		$(#[$attr])*
		#[derive(Debug)]
		pub struct $name<T>(pub T);

		impl<T> $crate::Container for $name<T> {
			type Inner = T;

			fn from_inner(inner: Self::Inner) -> Self {
				Self(inner)
			}

			fn get_ref(&self) -> &Self::Inner {
				&self.0
			}

			fn get_mut(&mut self) -> &mut Self::Inner {
				&mut self.0
			}

			fn into_inner(self) -> Self::Inner {
				self.0
			}

		}
	};
}

macro_rules! impl_newtype_read {
	($name:ident: $($bounds:tt)*) => {
		impl<T> $crate::IntoRead<$name<T>> for T
		where
			T: $($bounds)*
		{
			fn into_read(self) -> $name<T> {
				$name(self)
			}
		}
	};
}

macro_rules! impl_newtype_write {
	($name:ident: $($bounds:tt)*) => {
		impl<T> $crate::IntoWrite<$name<T>> for T
		where
			T: $($bounds)*
		{
			fn into_write(self) -> $name<T> {
				$name(self)
			}
		}
	};
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
}

macro_rules! if_feature {
	(if $feature:literal {
		$($item:item)*
	}) => {
		$(
			#[cfg(feature = $feature)]
			$item
		)*
	};
}

mod native;
pub use self::native::{Native, NativeAsync};

if_feature! {
	if "std" {
		mod std;
		pub use self::std::Std;
	}
}

if_feature! {
	if "tokio" {
		mod tokio;
		pub use self::tokio::Tokio;
	}
}

if_feature! {
	if "futures" {
		mod futures;
		pub use self::futures::Futures;
	}
}

if_feature! {
	if "core2" {
		mod core2;
		pub use self::core2::Core2;
	}
}
