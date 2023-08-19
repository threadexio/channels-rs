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

	macro_rules! cfg_json {
		($($item:item)*) => {
			$(
				#[cfg(feature = "json")]
				$item
			)*
		}
	}
}

macro_rules! cfg_flate2 {
	($($item:item)*) => {
		$(
			#[cfg(feature = "flate2")]
			$item
		)*
	}
}

macro_rules! cfg_crc {
	($($item:item)*) => {
		$(
			#[cfg(feature = "crc")]
			$item
		)*
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
