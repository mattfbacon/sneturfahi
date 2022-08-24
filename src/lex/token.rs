//! The token type [Token] and the token kind type [Selmaho].
//!
//! [Token] is the type yielded by [lex] (well, actually, a `Result` where the `Ok` type is [Token]).
//!
//! [lex]: crate::lex::lex

use crate::span::Span;

/// The token yielded by the [`lex`] iterator.
///
/// Note: this type implements [`PartialEq`] and [`Eq`], but they compare the spans of the tokens, not the tokens themselves.
///
/// [lex]: crate::lex::lex
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
	/// For cmavo, if that cmavo is experimental.
	pub experimental: bool,
	/// The type of the token.
	pub selmaho: crate::lex::Selmaho,
	/// The position of the token within the input. This is also used to get the actual content of the token.
	pub span: Span,
}
