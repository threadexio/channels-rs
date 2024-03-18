use std::collections::HashMap;

use proc_macro::{
	Delimiter, Group, Punct, Spacing, TokenStream, TokenTree,
};

use crate::util::{
	try_parse_group_expected, try_parse_ident,
	try_parse_ident_expected, try_parse_punct_expected,
	TryParseError,
};

mod translate;
use self::translate::Translate;

pub fn entry(item: TokenStream) -> TokenStream {
	let mut item = item.into_iter();

	let replace = try_parse_replace_section(&mut item);
	let code = try_parse_code_section(&mut item);

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

	for token in input {
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

fn try_parse_code_section<I>(stream: I) -> TokenStream
where
	I: Iterator<Item = TokenTree>,
{
	let mut stream = stream.peekable();

	try_parse_ident_expected(&mut stream, "code")
		.expect("expected 'code'");
	try_parse_punct_expected(&mut stream, ':').expect("expected ':'");
	let code =
		try_parse_group_expected(&mut stream, Delimiter::Brace)
			.expect("expected '{}' group");

	code.stream()
}

fn try_parse_replace_section<I>(stream: I) -> Vec<Translate>
where
	I: Iterator<Item = TokenTree>,
{
	let mut stream = stream.peekable();

	try_parse_ident_expected(&mut stream, "replace")
		.expect("expected 'replace'");
	try_parse_punct_expected(&mut stream, ':').expect("expected ':'");

	let mut stream =
		try_parse_group_expected(&mut stream, Delimiter::Brace)
			.expect("expected '{}' group")
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
			Err(e) => panic!("{e:?}"),
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
				Err(e) => panic!("{e:?}"),
			}
			.stream()
			.into_iter()
			.peekable();

			let src = try_parse_ident(&mut stream)
				.expect("expected source identifier");

			check_spacing_joint(
				try_parse_punct_expected(&mut stream, '=')
					.expect("expected '=>'"),
			);
			try_parse_punct_expected(&mut stream, '>')
				.expect("expected '=>'");

			let dst = stream.collect::<TokenStream>();

			table.insert(src.to_string(), dst);
		}

		sets.push(Translate::new(table));
	}

	sets
}

fn check_spacing_joint(punct: Punct) -> Punct {
	match punct.spacing() {
		Spacing::Joint => punct,
		Spacing::Alone => panic!("punct spacing not joint"),
	}
}
