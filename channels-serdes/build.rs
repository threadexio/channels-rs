fn set_cfg(cfg: &str, value: Option<&str>) {
	match value {
		Some(v) => println!("cargo:rustc-cfg={cfg}={v}"),
		None => println!("cargo:rustc-cfg={cfg}"),
	}
}

fn needs_std() {
	set_cfg("needs_std", None);
}

fn main() {
	#[cfg(feature = "cbor")]
	needs_std();
}
