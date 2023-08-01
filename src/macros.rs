macro_rules! cfg_feature {
	($feature:literal, $($tail:tt)*) => {
		$(
			#[cfg(feature = $feature)]
			#[cfg_attr(docrs, doc(cfg(feature = $feature)))]
			$tail
		)*
	};
	($feature:literal, $code:block) => { #[cfg(feature = $feature)] $code };
}

macro_rules! cfg_statistics {
	($code:block) => { $crate::macros::cfg_feature!("statistics", $code); };
	($($item:item)*) => { $crate::macros::cfg_feature!("statistics", $($item)*); };
}

macro_rules! cfg_serde {
	($code:block) => { $crate::macros::cfg_feature!("serde", $code); };
	($($item:item)*) => { $crate::macros::cfg_feature!("serde", $($item)*); };
}

macro_rules! cfg_tokio {
	($code:block) => { $crate::macros::cfg_feature!("tokio", $code); };
	($($item:item)*) => { $crate::macros::cfg_feature!("tokio", $($item)*); };
}

pub(crate) use cfg_feature;
pub(crate) use cfg_serde;
pub(crate) use cfg_statistics;
pub(crate) use cfg_tokio;
