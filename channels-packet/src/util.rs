#[cfg(has_core_error)]
#[allow(unused)]
pub use core::error::Error;

#[cfg(all(not(has_core_error), feature = "std"))]
#[allow(unused)]
pub use std::error::Error;

#[cfg(all(not(has_core_error), not(feature = "std")))]
#[allow(unused)]
pub trait Error: core::fmt::Debug + core::fmt::Display {}
