use channels::{serdes::Json, Sender};
use serde::{Deserialize, Serialize};
use std::io::stdout;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Data {
	num: i32,
}

impl Data {
	pub fn random() -> Self {
		Self { num: rand::random() }
	}
}

fn main() {
	let mut tx = Sender::builder()
		.writer(stdout().lock())
		.serializer(Json::new())
		.build();

	loop {
		let data = Data::random();
		tx.send_blocking(data).unwrap();
		sleep(Duration::from_secs(2));
	}
}
