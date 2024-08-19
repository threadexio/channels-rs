use channels::{io::Std, Receiver, Sender};

#[allow(clippy::unusual_byte_groupings)]
const PACKET: &[u8] = &[
	//66, 0, 189, 255
	0x42,        // version
	0b000000_00, // frame_num (6 bits) + len_words (2 bits)
	0xbd,
	0xff, // checksum
];

#[test]
fn conformance_sender() {
	let buf: Vec<u8> = Vec::with_capacity(32);

	let mut s: Sender<(), Std<_>, _> = Sender::builder()
		.serializer(channels::serdes::Bincode::new())
		.writer(buf)
		.build();

	s.send_blocking(()).unwrap();
	assert_eq!(s.get(), PACKET);
}

#[test]
fn conformance_receiver() {
	let mut r: Receiver<(), Std<_>, _> = Receiver::builder()
		.deserializer(channels::serdes::Bincode::new())
		.reader(PACKET)
		.build();

	let _: () = r.recv_blocking().unwrap();
}
