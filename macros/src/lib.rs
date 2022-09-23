#![deny(
	absolute_paths_not_starting_with_crate,
	elided_lifetimes_in_paths,
	explicit_outlives_requirements,
	future_incompatible,
	keyword_idents,
	macro_use_extern_crate,
	meta_variable_misuse,
	missing_abi,
	non_ascii_idents,
	nonstandard_style,
	noop_method_call,
	pointer_structural_match,
	private_in_public,
	rust_2018_idioms,
	unused_qualifications
)]
#![warn(
	clippy::pedantic,
	missing_copy_implementations,
	missing_debug_implementations
)]
#![allow(
	clippy::tabs_in_doc_comments, // rustfmt formats our doc comments and we use tabs
	clippy::redundant_else, // sometimes it's clearer
)]
#![forbid(unsafe_code)]

use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

mod derive_parse;

#[proc_macro_error]
#[proc_macro_derive(Parse, attributes(cut, parse))]
pub fn derive_parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	derive_parse::derive_parse(input)
}

#[cfg(feature = "make-assert-parse-test")]
fn make_hash(raw: &str) -> u64 {
	use std::hash::{Hash as _, Hasher as _};
	let mut hasher = std::collections::hash_map::DefaultHasher::new();
	raw.hash(&mut hasher);
	hasher.finish()
}

#[cfg(feature = "make-assert-parse-test")]
#[proc_macro]
pub fn make_assert_parse_test(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = proc_macro2::TokenStream::from(input);
	let span = syn::spanned::Spanned::span(&input);
	let input = input.into();
	let lit: syn::LitStr = parse_macro_input!(input);
	let value = lit.value();
	let underscore_name = value[..std::cmp::min(value.len(), 20)]
		.replace('\'', "h")
		.replace(|ch: char| !ch.is_alphabetic(), "_");
	let hash = make_hash(&value);
	let ident = quote::format_ident!("assert_parse_{hash:016x}_{underscore_name}");
	quote::quote_spanned! {span=>
		#[test]
		fn #ident() {
			let sentence = #lit;
			eprintln!(".i caku jai cipra lodu'u gendra fa lu {:?} li'u", sentence);
			crate::Cst::parse(&crate::lex(sentence).collect::<Result<Vec<_>, _>>().expect("lexing failed")).expect("parsing failed");
		}
	}.into()
}

#[proc_macro_attribute]
pub fn debug_rule(
	_attrs: proc_macro::TokenStream,
	body: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	let ItemFn {
		attrs,
		vis,
		sig,
		block,
	} = parse_macro_input!(body as ItemFn);
	let name = syn::LitStr::new(&sig.ident.to_string(), proc_macro::Span::call_site().into());
	quote! {
		#(#attrs)*
		#vis #sig {
			debug_rule_start!(#name);
			let result = ((|input: &str| -> ParseResult<'_> #block) as for<'a> fn(&'a str) -> ParseResult<'a>)(input);
			debug_rule_end!(#name, result);
			result
		}
	}
	.into()
}
