use std::fmt::{self, Display};

use colored::Colorize;

#[derive(Debug)]
pub struct Address<D: Display>(pub D);

impl<D: Display> Display for Address<D> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.to_string().blue().bold().fmt(f)
	}
}

#[derive(Debug)]
pub struct Username<D: Display>(pub D);

impl<D: Display> Display for Username<D> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.to_string().bright_white().bold().fmt(f)
	}
}

#[derive(Debug)]
pub struct Message<D: Display>(pub D);

impl<D: Display> Display for Message<D> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}
