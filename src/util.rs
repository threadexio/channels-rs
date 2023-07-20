use core::cell::Cell;
use core::marker::PhantomData;

/// Marker type that implements `!Send` and `!Sync`.
/// Workaround for unimplemented negative trait impls.
pub type PhantomUnsend = PhantomData<*const ()>;

/// Marker type that implements `!Sync`.
/// Workaround for unimplemented negative trait impls.
pub type PhantomUnsync = PhantomData<Cell<()>>;

macro_rules! flags {
	(
		$(#[$attr:meta])*
		$vis:vis struct $name:ident ($t:ty) {
			$(
				const $flag:ident = $flag_value:expr;
			)*
		}
	) => {
		$(#[$attr])*
		$vis struct $name(pub $t);

		impl $name {
			$(
				pub const $flag: Self = Self($flag_value);
			)*

			pub fn zero() -> Self {
				Self(0)
			}
		}

		impl ::core::ops::BitAnd for $name {
			type Output = bool;

			fn bitand(self, rhs: Self) -> Self::Output {
				self.0 & rhs.0 != 0
			}
		}

		impl ::core::ops::BitOr for $name {
			type Output = Self;

			fn bitor(self, rhs: Self) -> Self::Output {
				Self(self.0 | rhs.0)
			}
		}

		impl ::core::ops::BitOrAssign for $name {
			fn bitor_assign(&mut self, rhs: Self) {
				self.0 |= rhs.0;
			}
		}

		impl ::core::ops::BitXor for $name {
			type Output = Self;

			fn bitxor(self, rhs: Self) -> Self::Output {
				Self(self.0 ^ rhs.0)
			}
		}

		impl ::core::ops::BitXorAssign for $name {
			fn bitxor_assign(&mut self, rhs: Self) {
				self.0 ^= rhs.0;
			}
		}
	};
}
pub(crate) use flags;
