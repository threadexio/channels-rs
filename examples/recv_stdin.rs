use channels::{serdes::Json, Receiver};
use serde::{Deserialize, Serialize};
use std::io::stdin;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Data {
	num: i32,
}

fn main() {
	let mut rx = Receiver::<Data, _, _>::builder()
		.reader(stdin().lock())
		.deserializer(Json::new())
		.build();

	loop {
		rx.recv_blocking().unwrap();
	}
}
