use std::cell::UnsafeCell;
use std::sync::Arc;

pub type Shared<T> = Outer<Inner<T>>;

pub type Outer<T> = Arc<T>;

pub struct Inner<T> {
	data: UnsafeCell<T>,
}

impl<T> Inner<T> {
	pub fn new(stream: T) -> Self {
		Self { data: UnsafeCell::new(stream) }
	}

	pub fn get(&self) -> &mut T {
		unsafe { self.data.get().as_mut().unwrap() }
	}
}
