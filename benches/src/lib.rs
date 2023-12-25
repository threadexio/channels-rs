use serde::{Deserialize, Serialize};

pub type Simple = i32;

#[derive(Debug, Serialize, Deserialize)]
pub struct Complex {
	pub a: i32,
	pub b: usize,
	pub c: String,
}

pub fn simple() -> Simple {
	-42
}

pub fn complex() -> Complex {
	Complex {
		a: 42,
		b: 0xbadbeef,
		c: "This is my beautiful long test string".into(),
	}
}
