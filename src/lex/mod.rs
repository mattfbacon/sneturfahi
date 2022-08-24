//! Tokenization of Lojban text, including handling of delimited and pause-delimited quotes.
//!
//! This module centers around the [lex] function, which is heavily documented.

use std::num::NonZeroU8;

use crate::span::Span;

pub mod selmaho;
pub mod token;

pub use selmaho::Selmaho;
pub use token::Token;

/// Reasons why lexing can fail.
///
/// If an error occurs, lexing will terminate and you should not continue calling `next` on the lexer.
#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum Error {
	/// Expected a separator after delimited quote initiator but found the end of input.
	///
	/// ```rust
	/// # use sneturfahi::lex::{lex, Error};
	/// let invalid_input = "zoi";
	/// let result = lex(invalid_input).collect::<Result<Vec<_>, _>>();
	/// assert!(matches!(
	/// 	result,
	/// 	Err(Error::DelimitedQuoteMissingSeparator { .. })
	/// ));
	/// ```
	#[error("expected a separator after a delimited quote initiator but found the end of input")]
	DelimitedQuoteMissingSeparator {
		/// The span of the word that initiated the delimited quote.
		///
		/// ```rust
		/// # use sneturfahi::lex::{lex, Error};
		/// let invalid_input = "zoi";
		/// let error = lex(invalid_input)
		/// 	.collect::<Result<Vec<_>, _>>()
		/// 	.unwrap_err();
		/// if let Error::DelimitedQuoteMissingSeparator { initiator_span } = error {
		/// 	assert_eq!(initiator_span.slice(invalid_input).unwrap(), "zoi");
		/// } else {
		/// 	unreachable!("expected DelimitedQuoteMissingSeparator variant, got {error:?}");
		/// }
		/// ```
		initiator_span: Span,
	},
	/// A delimited quote is unclosed.
	///
	/// ```rust
	/// # use sneturfahi::lex::{lex, Error};
	/// let invalid_input = "zoi gy my nice text but no terminator";
	/// let result = lex(invalid_input).collect::<Result<Vec<_>, _>>();
	/// assert!(matches!(result, Err(Error::DelimitedQuoteUnclosed { .. })));
	/// ```
	#[error("a delimited quote is unclosed")]
	DelimitedQuoteUnclosed {
		/// The span of the word that initiated the delimited quote.
		///
		/// ```rust
		/// # use sneturfahi::lex::{lex, Error};
		/// let invalid_input = "zoi gy my nice text but no terminator";
		/// let error = lex(invalid_input)
		/// 	.collect::<Result<Vec<_>, _>>()
		/// 	.unwrap_err();
		/// if let Error::DelimitedQuoteUnclosed { initiator_span, .. } = error {
		/// 	assert_eq!(initiator_span.slice(invalid_input).unwrap(), "zoi");
		/// } else {
		/// 	unreachable!("expected DelimitedQuoteUnclosed variant, got {error:?}");
		/// }
		/// ```
		initiator_span: Span,
		/// The span of the starting delimiter, of which the matching ending delimiter was not found.
		///
		/// ```rust
		/// # use sneturfahi::lex::{lex, Error};
		/// let invalid_input = "zoi gy my nice text but no terminator";
		/// let error = lex(invalid_input)
		/// 	.collect::<Result<Vec<_>, _>>()
		/// 	.unwrap_err();
		/// if let Error::DelimitedQuoteUnclosed {
		/// 	starting_delimiter_span,
		/// 	..
		/// } = error
		/// {
		/// 	assert_eq!(starting_delimiter_span.slice(invalid_input).unwrap(), "gy");
		/// } else {
		/// 	unreachable!("expected DelimitedQuoteUnclosed variant, got {error:?}");
		/// }
		/// ```
		starting_delimiter_span: Span,
	},
	/// Expected the content of a pause-delimited quote but found the end of input.
	///
	/// ```rust
	/// # use sneturfahi::lex::{lex, Error};
	/// let invalid_input = "me'oi";
	/// let result = lex(invalid_input).collect::<Result<Vec<_>, _>>();
	/// assert!(matches!(result, Err(Error::PauseDelimitedQuoteEof { .. })));
	/// ```
	#[error("expected the content of a pause-delimited quote but found the end of input")]
	PauseDelimitedQuoteEof {
		/// The span of the word that initiated the pause-delimited quote.
		///
		/// ```rust
		/// # use sneturfahi::lex::{lex, Error};
		/// let invalid_input = "me'oi";
		/// let error = lex(invalid_input)
		/// 	.collect::<Result<Vec<_>, _>>()
		/// 	.unwrap_err();
		/// if let Error::PauseDelimitedQuoteEof { initiator_span } = error {
		/// 	assert_eq!(initiator_span.slice(invalid_input).unwrap(), "me'oi");
		/// } else {
		/// 	unreachable!("expected PauseDelimitedQuoteEof variant, got {error:?}");
		/// }
		/// ```
		initiator_span: Span,
	},
}

/// A [`Result`] where the `E` type defaults to [`Error`].
///
/// [Result]: std::result::Result
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Copy)]
struct DelimitedQuoteState {
	how_many: NonZeroU8,
	initiator_span: Span,
	is_first: bool,
	starting_delimiter_span: Span,
}

impl DelimitedQuoteState {
	fn next(self, ending_delimiter_span: Span) -> Option<Self> {
		self
			.how_many
			.get()
			.checked_sub(1)
			.and_then(NonZeroU8::new)
			.map(|new_how_many| Self {
				how_many: new_how_many,
				initiator_span: self.initiator_span,
				is_first: false,
				starting_delimiter_span: ending_delimiter_span,
			})
	}
}

enum State {
	Normal,
	DelimitedQuote(DelimitedQuoteState),
	TwoMoreTokensThen([Token; 2], Option<DelimitedQuoteState>),
	OneMoreTokenThen(Token, Option<DelimitedQuoteState>),
	PauseDelimitedQuote { initiator_span: Span },
	Done,
}

struct Lexer<'input> {
	words: crate::decompose::Decomposer<'input>,
	input: &'input str,
	state: State,
}

#[allow(clippy::unnecessary_wraps)] // consistency
impl Lexer<'_> {
	fn next_normal(&mut self) -> Option<Result<Token>> {
		let span = self.words.next()?;
		let word = span.slice(self.input).unwrap();
		let (selmaho, experimental) = Selmaho::classify(word);
		match selmaho {
			Selmaho::Zoi | Selmaho::Muhoi | Selmaho::Sohehai => {
				let how_many = if selmaho == Selmaho::Sohehai { 2 } else { 1 };
				self.state = State::DelimitedQuote(DelimitedQuoteState {
					how_many: NonZeroU8::new(how_many).unwrap(),
					initiator_span: span,
					is_first: true,
					starting_delimiter_span: if let Some(span) = self.words.next() {
						span
					} else {
						self.state = State::Done;
						return Some(Err(Error::DelimitedQuoteMissingSeparator {
							initiator_span: span,
						}));
					},
				});
			}
			Selmaho::Mehoi | Selmaho::Zohoi | Selmaho::Dohoi => {
				self.state = State::PauseDelimitedQuote {
					initiator_span: span,
				};
			}
			Selmaho::Faho | Selmaho::Fahoho => {
				self.state = State::Done;
			}
			_ => (),
		}
		Some(Ok(Token {
			experimental,
			selmaho,
			span,
		}))
	}

	fn next_delimited_quote(
		&mut self,
		quote_state @ DelimitedQuoteState {
			is_first,
			starting_delimiter_span,
			initiator_span,
			how_many: _,
		}: DelimitedQuoteState,
	) -> Option<Result<Token>> {
		let mut start_of_quote = None;
		let mut end_of_quote = None;
		Some(loop {
			if let Some(word_span) = self.words.next_no_decomposition() {
				let possible_ending_delimiter = word_span.slice(self.input).unwrap();
				let starting_delimiter = starting_delimiter_span.slice(self.input).unwrap();
				if starting_delimiter
					.chars()
					.filter(|&ch| ch != ',')
					.eq(possible_ending_delimiter.chars().filter(|&ch| ch != ','))
				{
					let ending_delimiter_span = word_span;
					let start_token = Token {
						experimental: false,
						span: starting_delimiter_span,
						selmaho: Selmaho::ZoiDelimiter,
					};
					let text_token = (|| {
						Some(Token {
							experimental: false,
							span: Span::new(start_of_quote?, end_of_quote?),
							selmaho: Selmaho::AnyText,
						})
					})();
					let end_token = Token {
						experimental: false,
						span: ending_delimiter_span,
						selmaho: Selmaho::ZoiDelimiter,
					};
					let next_quote_state = quote_state.next(ending_delimiter_span);
					if is_first {
						self.state = text_token.map_or(
							State::OneMoreTokenThen(end_token, next_quote_state),
							|text_token| State::TwoMoreTokensThen([text_token, end_token], next_quote_state),
						);
						break Ok(start_token);
					} else if let Some(text_token) = text_token {
						self.state = State::OneMoreTokenThen(end_token, next_quote_state);
						break Ok(text_token);
					} else {
						self.state = next_quote_state.map_or(State::Normal, State::DelimitedQuote);
						break Ok(end_token);
					}
				} else {
					let quote_part = word_span;
					start_of_quote.get_or_insert(quote_part.start);
					end_of_quote = Some(quote_part.end);
				}
			} else {
				self.state = State::Done;
				break Err(Error::DelimitedQuoteUnclosed {
					initiator_span,
					starting_delimiter_span,
				});
			}
		})
	}

	fn next_pause_delimited_quote(&mut self, initiator_span: Span) -> Option<Result<Token>> {
		let quoted_text_span = if let Some(word) = self.words.next_no_decomposition() {
			word
		} else {
			self.state = State::Done;
			return Some(Err(Error::PauseDelimitedQuoteEof { initiator_span }));
		};
		self.state = State::Normal;
		Some(Ok(Token {
			experimental: false,
			selmaho: Selmaho::AnyText,
			span: quoted_text_span,
		}))
	}
}

impl Iterator for Lexer<'_> {
	type Item = Result<Token>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.state {
			State::Normal => self.next_normal(),
			State::DelimitedQuote(quote_state) => self.next_delimited_quote(quote_state),
			State::TwoMoreTokensThen([text_token, end_token], then) => {
				self.state = State::OneMoreTokenThen(end_token, then);
				Some(Ok(text_token))
			}
			State::OneMoreTokenThen(end_token, then) => {
				self.state = then.map_or(State::Normal, State::DelimitedQuote);
				Some(Ok(end_token))
			}
			State::PauseDelimitedQuote { initiator_span } => {
				self.next_pause_delimited_quote(initiator_span)
			}
			State::Done => None,
		}
	}
}

impl std::iter::FusedIterator for Lexer<'_> {}

#[allow(clippy::doc_markdown)] // it incorrectly flags selmaho like MEhOI as code
/// Lex the Lojban text into a sequence of [`Token`]s, with an [`Error`] possibly occurring.
///
/// Typical usage of this function involves collecting to `Result<Vec<_>, _>`, where the first `_` will be inferred to be `Token` and the second `Error`:
///
/// ```rust
/// # use sneturfahi::lex::{lex, Token, Selmaho};
/// # use sneturfahi::span::Span;
/// let input = "mi nelci la sneturfa'i";
/// let result: Result<Vec<_>, _> = lex(input).collect();
/// assert_eq!(
/// 	result.unwrap(),
/// 	[
/// 		Token {
/// 			experimental: false,
/// 			selmaho: Selmaho::Koha,
/// 			span: Span::new(0, 2),
/// 		},
/// 		Token {
/// 			experimental: false,
/// 			selmaho: Selmaho::Gismu,
/// 			span: Span::new(3, 8),
/// 		},
/// 		Token {
/// 			experimental: false,
/// 			selmaho: Selmaho::La,
/// 			span: Span::new(9, 11),
/// 		},
/// 		Token {
/// 			experimental: false,
/// 			selmaho: Selmaho::Lujvo,
/// 			span: Span::new(12, 22),
/// 		},
/// 	]
/// );
/// ```
///
/// However, as shown in the following examples, one doesn't typically process the lexer output wholesale.
///
/// This function has special processing for words that quote non-Lojban text, such as `zoi` and `me'oi`:
///
/// ```rust
/// # use sneturfahi::lex::lex;
/// let input = "zoi gy hello world gy me'oi alobroda";
/// let result: Result<Vec<_>, _> = lex(input)
/// 	.map(|token| token.map(|token| token.span.slice(input).unwrap()))
/// 	.collect();
/// // "hello world" and "alobroda" are grouped into single AnyText tokens.
/// assert_eq!(
/// 	result.unwrap(),
/// 	["zoi", "gy", "hello world", "gy", "me'oi", "alobroda"]
/// );
/// ```
///
/// For selmaho like ZOI and MUhOI which initiate delimited quotes, the selmaho of the tokens yielded after will be `[ZoiDelimiter, AnyText, ZoiDelimiter]`:
///
/// ```rust
/// # use sneturfahi::lex::{lex, Selmaho};
/// let input = "zoi gy hello world gy";
/// let result: Result<Vec<_>, _> = lex(input)
/// 	.map(|token| token.map(|token| token.selmaho))
/// 	.collect();
/// assert_eq!(
/// 	result.unwrap(),
/// 	[
/// 		Selmaho::Zoi,
/// 		Selmaho::ZoiDelimiter,
/// 		Selmaho::AnyText,
/// 		Selmaho::ZoiDelimiter
/// 	]
/// );
/// ```
///
/// For selmaho like SOhEhAI which initiate multiple delimited quotes, the text chunks of the quotes will be surrounded and separated by `ZoiDelimiter`s:
///
/// ```rust
/// # use sneturfahi::lex::{lex, Selmaho};
/// let input = "so'e'ai gy cipna gy sipna gy";
/// let result: Result<Vec<_>, _> = lex(input)
/// 	.map(|token| token.map(|token| token.selmaho))
/// 	.collect();
/// assert_eq!(
/// 	result.unwrap(),
/// 	[
/// 		Selmaho::Sohehai,
/// 		Selmaho::ZoiDelimiter,
/// 		Selmaho::AnyText,
/// 		Selmaho::ZoiDelimiter,
/// 		Selmaho::AnyText,
/// 		Selmaho::ZoiDelimiter
/// 	]
/// );
/// ```
///
/// For selmaho like MEhOI, ZOhOI, and DOhOI which initiate pause-delimited quotes, the text will be yielded as a single `AnyText` token:
///
/// ```rust
/// # use sneturfahi::lex::{lex, Selmaho};
/// let input = "me'oi broda";
/// let result: Result<Vec<_>, _> = lex(input)
/// 	.map(|token| token.map(|token| token.selmaho))
/// 	.collect();
/// assert_eq!(result.unwrap(), [Selmaho::Mehoi, Selmaho::AnyText]);
/// ```
///
/// The lexer also propagates the feature of `Selmaho::classify` of identifying experimental cmavo, through the `experimental` field of the yielded [`Token`]s:
///
/// ```rust
/// # use sneturfahi::lex::{lex, Selmaho};
/// let input = "ui ca'e'ei i'au";
/// let result: Result<Vec<_>, _> = lex(input)
/// 	.map(|token| token.map(|token| token.experimental))
/// 	.collect();
/// assert_eq!(result.unwrap(), [false, true, true]);
/// ```
///
/// After an `Err` variant is yielded, the lexer iterator is fused, that is, it will yield `None` indefinitely. This may help with certain aspects of implementations.
pub fn lex(input: &str) -> impl Iterator<Item = Result<Token>> + std::iter::FusedIterator + '_ {
	Lexer {
		words: crate::decompose(input),
		input,
		state: State::Normal,
	}
}

#[cfg(test)]
mod test {
	macro_rules! make_test {
		($name:ident, $raw:expr, [$($ttype:ident($text:expr)),* $(,)?]) => {
			#[test]
			fn $name() {
				let raw = $raw;
				let result: Vec<_> = super::lex(raw)
					.map(Result::unwrap)
					.map(|token| (token.selmaho, token.span.slice(raw).unwrap()))
					.collect();
				assert_eq!(result, &[$((super::selmaho::Selmaho::$ttype, $text),)*] as &[(super::selmaho::Selmaho, &str)]);
			}
		};
		($name:ident, $raw:expr, $actual:expr) => {
			use crate::Span;
			use crate::lex::token::Token;
			use crate::lex::selmaho::Selmaho::*;
			#[test]
			fn $name() {
				let raw = $raw;
				let result: Vec<_> = super::lex(raw)
					.map(Result::unwrap)
					.collect();
				assert_eq!(result, $actual);
			}
		};
	}
	macro_rules! tests {
		() => {};
		($name:ident : $raw:expr => [$($expected:tt)*], $($rest:tt)*) => {
			make_test!($name, $raw, [$($expected)*]);
			tests!($($rest)*);
		};
		($name:ident : $raw:expr => $actual:expr, $($rest:tt)*) => {
			make_test!($name, $raw, $actual);
			tests!($($rest)*);
		}
	}

	tests! {
		// copy the behavior of other parsers
		do_decompose_starting_delimiter: "zoi fuvi text fu" => [Zoi("zoi"), ZoiDelimiter("fu"), AnyText("vi text"), ZoiDelimiter("fu")],
		dont_decompose_in_quotes: "zoi gy gygy gy" => [Zoi("zoi"), ZoiDelimiter("gy"), AnyText("gygy"), ZoiDelimiter("gy")],
		basic: "fuvi zoi gy broda gy fuvi" => [Fa("fu"), Va("vi"), Zoi("zoi"), ZoiDelimiter("gy"), AnyText("broda"), ZoiDelimiter("gy"), Fa("fu"), Va("vi")],
		pause_delimited_quotes: "zo'oi abcdef la'oi abcdef me'oi abcdef ra'oi abcdef do'oi abcdef" => [Zohoi("zo'oi"), AnyText("abcdef"), Zohoi("la'oi"), AnyText("abcdef"), Mehoi("me'oi"), AnyText("abcdef"), Zohoi("ra'oi"), AnyText("abcdef"), Dohoi("do'oi"), AnyText("abcdef")],
		pause_delimited_delimited_by_actual_pauses: "zo'oi.abcdef.la'oi.abcdef." => [Zohoi("zo'oi"), AnyText("abcdef"), Zohoi("la'oi"), AnyText("abcdef")],
		sohehai: "so'e'ai gy. cipna .gy. sipna .gy" => [Sohehai("so'e'ai"), ZoiDelimiter("gy"), AnyText(" cipna "), ZoiDelimiter("gy"), AnyText(" sipna "), ZoiDelimiter("gy")],
		empty_quote: "zoi gy gy" => [Zoi("zoi"), ZoiDelimiter("gy"), ZoiDelimiter("gy")],
		empty_quote2: "zoi gy.gy" => [Zoi("zoi"), ZoiDelimiter("gy"), ZoiDelimiter("gy")],
		whitespace_rules: "zoi gy no pauses on the delimiters gy" => [Zoi("zoi"), ZoiDelimiter("gy"), AnyText("no pauses on the delimiters"), ZoiDelimiter("gy")],
		whitespace_rules2: "zoi gy. pause at start gy" => [Zoi("zoi"), ZoiDelimiter("gy"), AnyText(" pause at start"), ZoiDelimiter("gy")],
		whitespace_rules3: "zoi gy pause at end .gy" => [Zoi("zoi"), ZoiDelimiter("gy"), AnyText("pause at end "), ZoiDelimiter("gy")],
		whitespace_rules4: "zoi gy. pauses on both .gy" => [Zoi("zoi"), ZoiDelimiter("gy"), AnyText(" pauses on both "), ZoiDelimiter("gy")],
		whitespace_rules5: "zoi gy   gy" => [Zoi("zoi"), ZoiDelimiter("gy"), AnyText(" "), ZoiDelimiter("gy")],
		whitespace_rules6: "zoi gy. . . . .gy" => [Zoi("zoi"), ZoiDelimiter("gy"), AnyText(" . . . "), ZoiDelimiter("gy")],
		srasu: include_str!("../srasu.txt") => include!("srasu.txt.expected"),
	}
}
