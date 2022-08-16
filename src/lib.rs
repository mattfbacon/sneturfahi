#![doc = include_str!("../README.md")]
#![feature(type_alias_impl_trait)]
#![deny(
	absolute_paths_not_starting_with_crate,
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
	rust_2018_idioms
)]
#![warn(clippy::pedantic, missing_docs, missing_copy_implementations)]
#![allow(
	clippy::tabs_in_doc_comments, // rustfmt formats our doc comments and we use tabs
	clippy::redundant_else, // sometimes it's clearer
)]
#![forbid(unsafe_code)]

mod rules;

pub mod decompose;
pub use decompose::decompose;

pub mod lex;
pub use lex::lex;

// pub mod parse;
// pub use parse::parse;

pub mod span;
pub use span::Span;
