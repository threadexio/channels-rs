use channels::{Receiver, Sender};

const PACKET: &[u8] = &[
	0xfd, 0x3f, // version
	0x00, 0x08, // length
	0x02, 0xb8, // checksum
	0x00, // flags
	0x00, // id
];

#[test]
fn conformance_sender() {
	let mut buf: Vec<u8> = Vec::with_capacity(32);

	let mut s = Sender::builder()
		.serializer(channels::serdes::Bincode::new())
		.writer(&mut buf)
		.build();

	s.send_blocking(()).unwrap();
	assert_eq!(&s.get()[..], PACKET);
}

#[test]
fn conformance_receiver() {
	let mut r = Receiver::builder()
		.deserializer(channels::serdes::Bincode::new())
		.reader(PACKET)
		.build();

	let _: () = r.recv_blocking().unwrap();
}
