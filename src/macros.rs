macro_rules! cfg_statistics {
	($($item:item)*) => {
		#[cfg(feature = "statistics")]
		#[cfg_attr(docrs, doc(cfg(feature = "statistics")))]
		$($item)*
	};
}
pub(crate) use cfg_statistics;

macro_rules! cfg_serde {
	($($item:item)*) => {
		#[cfg(feature = "serde")]
		#[cfg_attr(docrs, doc(cfg(feature = "serde")))]
		$($item)*
	};
}
pub(crate) use cfg_serde;
macro_rules! cfg_tokio {
	($($item:item)*) => {
		#[cfg(feature = "tokio")]
		#[cfg_attr(docrs, doc(cfg(feature = "tokio")))]
		$($item)*
	};
}
pub(crate) use cfg_tokio;
