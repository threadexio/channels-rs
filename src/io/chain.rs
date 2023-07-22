use super::{Result, Write};

pub struct Chain<T, U> {
	first: T,
	second: U,
	done_first: bool,
}

impl<T, U> Chain<T, U> {
	pub fn new(first: T, second: U) -> Self {
		Self { first, second, done_first: false }
	}

	pub fn into_inner(self) -> (T, U) {
		(self.first, self.second)
	}
}

impl<T, U> Write for Chain<T, U>
where
	T: Write,
	U: Write,
{
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		let mut i = 0;

		if !self.done_first {
			i += self.first.write(buf)?;
		}

		if i < buf.len() {
			self.done_first = true;

			i += self.second.write(&buf[i..])?;
		}

		Ok(i)
	}

	fn flush(&mut self) -> Result<()> {
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_chain_writer() {
		let mut a = [0u8; 4];
		let mut b = [0u8; 4];
		let mut w = Chain::new(&mut a[..], &mut b[..]);

		assert_eq!(w.write(&[1, 2]).unwrap(), 2);
		assert_eq!(w.write(&[3]).unwrap(), 1);
		assert_eq!(w.write(&[4, 5, 6]).unwrap(), 3);

		let _ = w.into_inner();

		assert_eq!(&a, &[1, 2, 3, 4]);
		assert_eq!(&b, &[5, 6, 0, 0]);
	}
}
