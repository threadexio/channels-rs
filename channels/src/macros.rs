macro_rules! cfg_statistics {
	($($item:item)*) => {
		$(
			#[cfg(feature = "statistics")]
			$item
		)*
	};
}

macro_rules! cfg_serde {
	($($item:item)*) => {
		$(
			#[cfg(feature = "serde")]
			$item
		)*
	};
}

cfg_serde! {
	macro_rules! cfg_bincode {
		($($item:item)*) => {
			$(
				#[cfg(feature = "bincode")]
				$item
			)*
		}
	}

	macro_rules! cfg_cbor {
		($($item:item)*) => {
			$(
				#[cfg(feature = "cbor")]
				$item
			)*
		}
	}
}

macro_rules! cfg_tokio {
	($($item:item)*) => {
		$(
			#[cfg(feature = "tokio")]
			$item
		)*
	};
}