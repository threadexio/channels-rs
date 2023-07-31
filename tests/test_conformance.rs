use channels::{Receiver, Sender};

#[test]
fn test_conformance_sender() {
	let mut buf: Vec<u8> = Vec::with_capacity(32);

	let mut s = Sender::<(), _, _>::new(&mut buf);

	s.send_blocking(()).unwrap();
	assert_eq!(
		&s.get()[..],
		&[
			0xfd, 0x3f, // version
			0x00, 0x08, // length
			0x02, 0xb8, // checksum
			0x00, // flags
			0x00, // id
		]
	);
}

#[test]
fn test_conformance_receiver() {
	let buf: &[u8] = &[
		0xfd, 0x3f, // version
		0x00, 0x08, // length
		0x02, 0xb8, // checksum
		0x00, // flags
		0x00, // id
	];

	let mut r = Receiver::<(), _, _>::new(buf);

	r.recv_blocking().unwrap();
}
