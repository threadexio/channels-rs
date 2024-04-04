use core::fmt::{self, Display};

use colored::Colorize;

pub fn setup_log(output: impl Into<fern::Output>) {
	use log::{Level, Record};

	fn record_to_prefix(record: &Record) -> impl fmt::Display {
		match record.level() {
			Level::Error => "error:".bright_red(),
			Level::Debug => "debug:".bright_white(),
			Level::Info => "info:".bright_green(),
			Level::Trace => "trace:".bright_black(),
			Level::Warn => "warn:".yellow(),
		}
		.bold()
	}

	colored::control::set_override(true);

	fern::Dispatch::new()
		.format(|out, message, record| {
			out.finish(format_args!(
				"{} {message}",
				record_to_prefix(record)
			));
		})
		.chain(output)
		.apply()
		.expect("failed to setup fern");
}

pub struct DisplayError<D: Display>(pub D);

impl<D: Display> Display for DisplayError<D> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.to_string().red().fmt(f)
	}
}

pub struct DisplayOk<D: Display>(pub D);

impl<D: Display> Display for DisplayOk<D> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.to_string().green().fmt(f)
	}
}

pub struct Tree<'a, 'b> {
	fmt: &'a mut fmt::Formatter<'b>,
	result: fmt::Result,
	nest: usize,
}

impl<'a, 'b> Tree<'a, 'b> {
	pub fn new(fmt: &'a mut fmt::Formatter<'b>, name: &str) -> Self {
		let result = writeln!(fmt, "╭──● {name}");
		Self { fmt, result, nest: 0 }
	}

	const NESTED_LEVEL_INDENT: &'static str = "│ ";

	fn add_nest_indent(&mut self) {
		self.result = self.result.and_then(|()| {
			(0..self.nest).try_for_each(|_| {
				self.fmt.write_str(Self::NESTED_LEVEL_INDENT)
			})
		});
	}

	pub fn field<F>(&mut self, name: &str, f: F) -> &mut Self
	where
		F: FnOnce(&mut fmt::Formatter) -> fmt::Result,
	{
		self.add_nest_indent();
		self.result = self.result.and_then(|()| {
			write!(self.fmt, "├─○ {:<1$} ", format!("{name}:"), 15)?;
			f(self.fmt)?;
			writeln!(self.fmt)
		});
		self
	}

	pub fn subtree(&mut self, name: &str) -> Tree<'_, 'b> {
		let result = writeln!(self.fmt, "├─┬──● {name}");
		Tree { fmt: self.fmt, result, nest: self.nest + 1 }
	}

	pub fn finish(&mut self) -> fmt::Result {
		self.add_nest_indent();
		self.result =
			self.result.and_then(|()| writeln!(self.fmt, "╰──●"));
		self.result
	}
}
