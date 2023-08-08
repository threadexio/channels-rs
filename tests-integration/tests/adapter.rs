#[derive(Debug)]
struct Dummy;

#[test]
fn adapter_api_unsync() {
	use channels::adapter::unsync::*;

	let (r, w) = split(Dummy);
	assert!(
		matches!(join(r, w), Ok(Dummy)),
		"failed to join r and w"
	);
}

#[test]
fn adapter_api_sync() {
	use channels::adapter::sync::*;

	let (r, w) = split(Dummy);
	assert!(
		matches!(join(r, w), Ok(Dummy)),
		"failed to join r and w"
	);
}
