use rand::Rng;
use serde::{Deserialize, Serialize};

fn random_str(len: usize) -> String {
	rand::thread_rng()
		.sample_iter(&rand::distributions::Alphanumeric)
		.take(len)
		.map(char::from)
		.collect()
}

fn random_friends() -> Vec<u64> {
	(0..5).map(|_| rand::random()).collect::<Vec<u64>>()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
	id: u64,
	name: String,
	friends: Vec<u64>,
}

impl Player {
	pub fn random() -> Self {
		Self {
			id: rand::random(),
			name: random_str(12),
			friends: random_friends(),
		}
	}
}
