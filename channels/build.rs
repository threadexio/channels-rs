#![allow(missing_docs)]

use core::str::from_utf8;
use std::env;
use std::process::Command;

fn main() {
	println!("cargo::rustc-check-cfg=cfg(has_core_error)");

	let minor_version = rust_minor_version();

	// `core::error::Error` was stabilized in 1.81.0
	// https://releases.rs/docs/1.81.0/#stabilized-apis
	if minor_version >= 81 {
		println!("cargo::rustc-cfg=has_core_error");
	}
}

fn rust_minor_version() -> u64 {
	let rustc =
		env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());

	let c = Command::new(rustc)
		.arg("--version")
		.output()
		.expect("failed to get rustc version");

	// "rustc 1.xx.x (xxxxxxxxx xxxx-xx-xx)"
	let output = from_utf8(&c.stdout)
		.expect("rustc did not output valid utf8");
	let mut parts = output.split(' ').skip(1);

	let version = parts.next().expect("expected rustc semver number");
	let mut semver_parts = version.split('.').skip(1);

	semver_parts
		.next()
		.expect("expected semver minor number")
		.parse()
		.expect("failed to parse semver minor number")
}
