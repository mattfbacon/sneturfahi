use std::num::NonZeroU8;

use crate::span::Span;

mod classify;
pub mod token;

pub use token::{Selmaho, Token};

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum Error {
	#[error("expected separator after delimited quote initiator but end of input found")]
	DelimitedQuoteMissingSeparator { initiator_span: Span },
	#[error("delimited quote is unclosed")]
	DelimitedQuoteUnclosed {
		initiator_span: Span,
		starting_delimiter_span: Span,
	},
	#[error("expected content of pause-delimited quote but end of input found")]
	PauseDelimitedQuoteEof { initiator_span: Span },
}

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
	Errored,
}

struct Lexer<'input> {
	words: crate::decompose::Decomposer<'input>,
	input: &'input str,
	state: State,
}

impl Iterator for Lexer<'_> {
	type Item = Result<Token, Error>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.state {
			State::Normal => {
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
							starting_delimiter_span: match self.words.next() {
								Some(span) => span,
								None => {
									self.state = State::Errored;
									return Some(Err(Error::DelimitedQuoteMissingSeparator {
										initiator_span: span,
									}));
								}
							},
						});
					}
					Selmaho::Mehoi | Selmaho::Zohoi | Selmaho::Dohoi => {
						self.state = State::PauseDelimitedQuote {
							initiator_span: span,
						};
					}
					_ => (),
				}
				Some(Ok(Token {
					selmaho,
					experimental,
					span,
				}))
			}
			State::DelimitedQuote(
				quote_state @ DelimitedQuoteState {
					is_first,
					starting_delimiter_span,
					initiator_span,
					how_many: _,
				},
			) => {
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
						self.state = State::Errored;
						break Err(Error::DelimitedQuoteUnclosed {
							initiator_span,
							starting_delimiter_span,
						});
					}
				})
			}
			State::TwoMoreTokensThen([text_token, end_token], then) => {
				self.state = State::OneMoreTokenThen(end_token, then);
				Some(Ok(text_token))
			}
			State::OneMoreTokenThen(end_token, then) => {
				self.state = then.map_or(State::Normal, State::DelimitedQuote);
				Some(Ok(end_token))
			}
			State::PauseDelimitedQuote { initiator_span } => {
				let quoted_text_span = match self.words.next_no_decomposition() {
					Some(word) => word,
					None => {
						self.state = State::Errored;
						return Some(Err(Error::PauseDelimitedQuoteEof { initiator_span }));
					}
				};
				self.state = State::Normal;
				Some(Ok(Token {
					experimental: false,
					selmaho: Selmaho::AnyText,
					span: quoted_text_span,
				}))
			}
			State::Errored => None,
		}
	}
}

impl std::iter::FusedIterator for Lexer<'_> {}

pub fn lex<'input, 'config>(
	input: &'input str,
) -> impl Iterator<Item = Result<Token, Error>> + std::iter::FusedIterator + 'input {
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
				assert_eq!(result, &[$((super::token::Selmaho::$ttype, $text),)*] as &[(super::token::Selmaho, &str)]);
			}
		};
	}
	macro_rules! tests {
		($($name:ident : $raw:expr => [$($expected:tt)*],)*) => {
			$(make_test!($name, $raw, [$($expected)*]);)*
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
	}
}
