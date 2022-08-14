#![feature(type_alias_impl_trait)]

mod rules;

pub mod decompose;
pub use decompose::decompose;

pub mod lex;
pub use lex::lex;

pub mod parse;
pub use parse::parse;

pub mod span;
pub use span::Span;
