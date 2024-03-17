use core::iter::Peekable;

use proc_macro::{Delimiter, Group, Ident, Punct, TokenTree};

/// Try to parse an identifier from `stream`.
///
/// This function will return [`None`] without advancing `stream` if an identifier
/// is not the next token of `stream`. Otherwise it will advance `stream` and
/// return the [`Ident`].
pub fn try_parse_ident<I>(stream: &mut Peekable<I>) -> Option<Ident>
where
	I: Iterator<Item = TokenTree>,
{
	let next = stream.peek()?;

	use TokenTree as TT;
	match next {
		TT::Ident(_) => {
			let TT::Ident(x) = stream.next().unwrap() else {
				panic!("peek() did not return the next element")
			};

			Some(x)
		},
		_ => None,
	}
}

/// Try to parse a punctuation from `stream`.
///
/// This function will return [`None`] without advancing `stream` if a punctuation
/// is not the next token of `stream`. Otherwise it will advance `stream` and
/// return the [`Punct`].
pub fn try_parse_punct<I>(stream: &mut Peekable<I>) -> Option<Punct>
where
	I: Iterator<Item = TokenTree>,
{
	let next = stream.peek()?;

	use TokenTree as TT;
	match next {
		TT::Punct(_) => {
			let TT::Punct(x) = stream.next().unwrap() else {
				panic!("peek() did not return the next element")
			};

			Some(x)
		},

		_ => None,
	}
}

/// Try to parse a group from `stream`.
///
/// This function will return [`None`] without advancing `stream` if a group
/// is not the next token of `stream`. Otherwise it will advance `stream` and
/// return the [`Group`].
pub fn try_parse_group<I>(stream: &mut Peekable<I>) -> Option<Group>
where
	I: Iterator<Item = TokenTree>,
{
	let next = stream.peek()?;

	use TokenTree as TT;
	match next {
		TT::Group(_) => {
			let TT::Group(x) = stream.next().unwrap() else {
				panic!("peek() did not return the next element")
			};

			Some(x)
		},
		_ => None,
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TryParseError {
	NotFound,
	NotExpected,
}

/// Try to parse an identifier from `stream` and fail if it is not `expected`.
pub fn try_parse_ident_expected<I>(
	stream: &mut Peekable<I>,
	expected: &str,
) -> Result<Ident, TryParseError>
where
	I: Iterator<Item = TokenTree>,
{
	let ident =
		try_parse_ident(stream).ok_or(TryParseError::NotFound)?;

	if ident.to_string() == expected {
		Ok(ident)
	} else {
		Err(TryParseError::NotExpected)
	}
}

/// Try to parse a punctuation from `stream` and fail if it is not `expected`.
pub fn try_parse_punct_expected<I>(
	stream: &mut Peekable<I>,
	expected: char,
) -> Result<Punct, TryParseError>
where
	I: Iterator<Item = TokenTree>,
{
	let punct =
		try_parse_punct(stream).ok_or(TryParseError::NotFound)?;

	if punct.as_char() == expected {
		Ok(punct)
	} else {
		Err(TryParseError::NotExpected)
	}
}

/// Try to parse a group from `stream` and fail if it is not `expected`.
pub fn try_parse_group_expected<I>(
	stream: &mut Peekable<I>,
	delimiter: Delimiter,
) -> Result<Group, TryParseError>
where
	I: Iterator<Item = TokenTree>,
{
	let group =
		try_parse_group(stream).ok_or(TryParseError::NotFound)?;

	if group.delimiter() == delimiter {
		Ok(group)
	} else {
		Err(TryParseError::NotExpected)
	}
}
