mod traits;

mod impls;

mod chain;
mod cursor;
mod limit;
mod take;

pub use self::chain::{chain, Chain};
pub use self::cursor::Cursor;
pub use self::limit::{limit, Limit};
pub use self::take::{take, Take};

pub use self::traits::*;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_buf() {
		let mut a: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

		assert_eq!(a.remaining(), 10);
		assert_eq!(a.chunk(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

		a.advance(4);
		assert_eq!(a.remaining(), 6);
		assert_eq!(a.chunk(), &[4, 5, 6, 7, 8, 9]);

		a.advance(6);
		assert_eq!(a.remaining(), 0);
		assert_eq!(a.chunk(), &[]);
	}

	#[test]
	fn test_chain() {
		let a: &[u8] = &[0, 1, 2, 3, 4];
		let b: &[u8] = &[5, 6, 7, 8, 9, 10];

		let mut buf = a.chain(b);

		assert_eq!(buf.remaining(), 11);
		buf.advance(3);
		assert_eq!(buf.remaining(), 8);
		assert_eq!(buf.chunk(), [3, 4].as_slice());

		{
			let mut chunks = buf.walk_chunks();
			assert_eq!(chunks.next(), Some([3, 4].as_slice()));
			assert_eq!(
				chunks.next(),
				Some([5, 6, 7, 8, 9, 10].as_slice())
			);
			assert_eq!(chunks.next(), None);
			assert_eq!(chunks.next(), None);
		}

		buf.advance(4);
		assert_eq!(buf.remaining(), 4);
		assert_eq!(buf.chunk(), [7, 8, 9, 10].as_slice());

		{
			let mut chunks = buf.walk_chunks();
			assert_eq!(chunks.next(), Some([7, 8, 9, 10].as_slice()));
			assert_eq!(chunks.next(), None);
			assert_eq!(chunks.next(), None);
		}

		buf.advance(4);
		assert_eq!(buf.remaining(), 0);
		assert_eq!(buf.chunk(), &[]);

		{
			let mut chunks = buf.walk_chunks();
			assert_eq!(chunks.next(), None);
		}
	}
}
