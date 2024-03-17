use std::collections::HashMap;

use proc_macro::{Ident, TokenStream, TokenTree};

#[derive(Debug)]
pub struct Translate {
	table: HashMap<String, TokenStream>,
}

impl Translate {
	pub fn new(table: HashMap<String, TokenStream>) -> Self {
		Self { table }
	}

	pub fn translate(&self, src: Ident) -> TokenStream {
		match self.table.get(&src.to_string()) {
			Some(x) => x
				.clone()
				.into_iter()
				.map(|mut x| {
					x.set_span(src.span());
					x
				})
				.collect(),
			None => TokenStream::from(TokenTree::Ident(src)),
		}
	}
}
