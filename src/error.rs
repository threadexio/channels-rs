#![allow(unused_macros)]
#![allow(unused_imports)]

use std::io;

pub type Error = io::Error;
pub type ErrorKind = io::ErrorKind;
pub type Result<T> = io::Result<T>;

pub enum ChannelError {
	ObjectTooLarge,
}

impl std::fmt::Display for ChannelError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use ChannelError::*;

		match self {
			ObjectTooLarge => write!(f, "object too large"),
		}
	}
}

macro_rules! other {
	($($arg:tt)*) => {
		std::io::Error::new(std::io::ErrorKind::Other, format!($($arg)*))
	};
}
pub(crate) use other;

macro_rules! error {
	($arg:expr) => {
		std::io::Error::new(std::io::ErrorKind::Other, format!("{}", $arg))
	};
}
pub(crate) use error;
