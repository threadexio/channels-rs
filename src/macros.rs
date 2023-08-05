macro_rules! cfg_statistics {
	($($item:item)*) => {
		$(
			#[cfg(feature = "statistics")]
			#[cfg_attr(docsrs, doc(cfg(feature = "statistics")))]
			$item
		)*
	};
}

macro_rules! cfg_serde {
	($($item:item)*) => {
		$(
			#[cfg(feature = "serde")]
			#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
			$item
		)*
	};
}

cfg_serde! {
	macro_rules! cfg_bincode {
		($($item:item)*) => {
			$(
				#[cfg(feature = "bincode")]
				#[cfg_attr(docsrs, doc(cfg(feature = "bincode")))]
				$item
			)*
		}
	}
}

macro_rules! cfg_tokio {
	($($item:item)*) => {
		$(
			#[cfg(feature = "tokio")]
			#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
			$item
		)*
	};
}
