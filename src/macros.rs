macro_rules! cfg_statistics {
	($($item:item)*) => {
		$(
			#[cfg(feature = "statistics")]
			#[cfg_attr(docrs, doc(cfg(feature = "statistics")))]
			$item
		)*
	};
}

macro_rules! cfg_serde {
	($($item:item)*) => {
		$(
			#[cfg(feature = "serde")]
			#[cfg_attr(docrs, doc(cfg(feature = "serde")))]
			$item
		)*
	};
}

macro_rules! cfg_tokio {
	($($item:item)*) => {
		$(
			#[cfg(feature = "tokio")]
			#[cfg_attr(docrs, doc(cfg(feature = "tokio")))]
			$item
		)*
	};
}
