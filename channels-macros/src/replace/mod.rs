use std::collections::HashMap;

use proc_macro::{
	Delimiter, Group, Punct, Spacing, TokenStream, TokenTree,
};

use crate::util::*;

mod translate;
use self::translate::Translate;

pub fn entry(item: TokenStream) -> TokenStream {
	let mut item = item.into_iter();

	let replace = try_parse_replace_section(&mut item).unwrap();
	let code = try_parse_code_section(&mut item).unwrap();

	replace
		.iter()
		.map(|r| process_token_tree(r, code.clone()))
		.fold(TokenStream::new(), |mut out, x| {
			out.extend(x);
			out
		})
}

fn process_token_tree(
	translate: &Translate,
	input: TokenStream,
) -> TokenStream {
	use TokenTree as TT;

	let mut output = TokenStream::new();

	for token in input.into_iter() {
		match token {
			TT::Group(x) => {
				let delimiter = x.delimiter();
				let span = x.span();
				let stream =
					process_token_tree(translate, x.stream());

				let mut group = Group::new(delimiter, stream);
				group.set_span(span);

				output.extend([TT::Group(group)]);
			},
			TT::Ident(x) => {
				let r = translate.translate(x);
				output.extend(r);
			},
			x => output.extend([x]),
		};
	}

	output
}

fn try_parse_code_section<I>(stream: I) -> Result<TokenStream, String>
where
	I: Iterator<Item = TokenTree>,
{
	let mut stream = stream.peekable();

	try_parse_ident_expected(&mut stream, "code").unwrap();
	try_parse_punct_expected(&mut stream, ':').unwrap();
	let code =
		try_parse_group_expected(&mut stream, Delimiter::Brace)
			.unwrap();
	Ok(code.stream())
}

fn try_parse_replace_section<I>(
	stream: I,
) -> Result<Vec<Translate>, String>
where
	I: Iterator<Item = TokenTree>,
{
	let mut stream = stream.peekable();

	try_parse_ident_expected(&mut stream, "replace").unwrap();
	try_parse_punct_expected(&mut stream, ':').unwrap();

	let mut stream =
		try_parse_group_expected(&mut stream, Delimiter::Brace)
			.unwrap()
			.stream()
			.into_iter()
			.peekable();

	let mut sets = Vec::new();

	loop {
		let mut stream = match try_parse_group_expected(
			&mut stream,
			Delimiter::Bracket,
		) {
			Ok(x) => x,
			Err(TryParseError::NotFound) => break,
			Err(e) => panic!("{:?}", e),
		}
		.stream()
		.into_iter()
		.peekable();

		let mut table = HashMap::new();

		loop {
			let mut stream = match try_parse_group_expected(
				&mut stream,
				Delimiter::Parenthesis,
			) {
				Ok(x) => x,
				Err(TryParseError::NotFound) => break,
				Err(e) => panic!("{:?}", e),
			}
			.stream()
			.into_iter()
			.peekable();

			let src = try_parse_ident(&mut stream).unwrap();

			check_spacing_joint(
				try_parse_punct_expected(&mut stream, '=').unwrap(),
			);
			try_parse_punct_expected(&mut stream, '>').unwrap();

			let dst = stream.collect::<TokenStream>();

			table.insert(src.to_string(), dst);
		}

		sets.push(Translate::new(table));
	}

	Ok(sets)
}

fn check_spacing_joint(punct: Punct) -> Punct {
	match punct.spacing() {
		Spacing::Joint => punct,
		Spacing::Alone => panic!("punct spacing not joint"),
	}
}
