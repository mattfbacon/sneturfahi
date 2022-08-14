use crate::span::Span;

mod classify;
pub mod config;
pub mod token;

pub use config::Config;
pub use token::{Selmaho, Token};

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum Error<'input> {
	#[error("expected separator after zoi but end of input found")]
	ZoiMissingSeparator { zoi_span: Span<'input> },
	#[error("zoi quote is unclosed")]
	ZoiUnclosed {
		zoi_span: Span<'input>,
		starting_delimiter_span: Span<'input>,
	},
}

type LexWords<'input> = impl Iterator<Item = Span<'input>>;

enum LexerState<'input> {
	Normal,
	ZoiQuote {
		zoi_span: Span<'input>,
		starting_delimiter_span: Span<'input>,
	},
	TwoMoreTokens([Token<'input>; 2]),
	OneMoreToken(Token<'input>),
}

struct Lexer<'input> {
	words: LexWords<'input>,
	input: &'input str,
	config: Config,
	state: LexerState<'input>,
}

impl<'input> Iterator for Lexer<'input> {
	type Item = Result<Token<'input>, Error<'input>>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.state {
			LexerState::Normal => {
				let span = self.words.next()?;
				let word = span.slice(self.input).unwrap();
				let (selmaho, experimental) = Selmaho::classify(word);
				match selmaho {
					Selmaho::Zoi => {
						self.state = LexerState::ZoiQuote {
							zoi_span: span,
							starting_delimiter_span: match self.words.next() {
								Some(span) => span,
								None => return Some(Err(Error::ZoiMissingSeparator { zoi_span: span })),
							},
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
			LexerState::ZoiQuote {
				starting_delimiter_span,
				zoi_span,
			} => {
				let mut words_no_decompose = starting_delimiter_span
					.slice_after(self.input)
					.unwrap()
					.split(crate::decompose::split_or_trim_condition)
					.filter(|chunk| !chunk.is_empty());
				Some(loop {
					if let Some(possible_ending_delimiter) = words_no_decompose.next() {
						if self.config.zoi_delimiter_comparison.compare(
							starting_delimiter_span.slice(self.input).unwrap(),
							possible_ending_delimiter,
						) {
							let ending_delimiter_span =
								Span::from_embedded_slice(self.input.as_ptr(), possible_ending_delimiter);
							let start_token = Token {
								experimental: false,
								span: starting_delimiter_span,
								selmaho: Selmaho::ZoiDelimiter,
							};
							let text_token = Token {
								experimental: false,
								span: Span::between(starting_delimiter_span, ending_delimiter_span),
								selmaho: Selmaho::AnyText,
							};
							let end_token = Token {
								experimental: false,
								span: ending_delimiter_span,
								selmaho: Selmaho::ZoiDelimiter,
							};
							self.state = LexerState::TwoMoreTokens([text_token, end_token]);
							self.input = ending_delimiter_span.slice_after(self.input).unwrap();
							self.words = crate::decompose(self.input);
							break Ok(start_token);
						}
					} else {
						break Err(Error::ZoiUnclosed {
							zoi_span,
							starting_delimiter_span,
						});
					}
				})
			}
			LexerState::TwoMoreTokens([text_token, end_token]) => {
				self.state = LexerState::OneMoreToken(end_token);
				Some(Ok(text_token))
			}
			LexerState::OneMoreToken(end_token) => {
				self.state = LexerState::Normal;
				Some(Ok(end_token))
			}
		}
	}
}

/// It is a logic error to call `next` on the resulting iterator after it has yielded `Result::Err`
pub fn lex<'input, 'config>(
	input: &'input str,
	config: Config,
) -> impl Iterator<Item = Result<Token<'input>, Error<'input>>> + 'input {
	Lexer {
		words: crate::decompose(input),
		input,
		config,
		state: LexerState::Normal,
	}
}

#[cfg(test)]
mod test {
	macro_rules! make_test {
		($name:ident, $raw:expr, [$($ttype:ident($text:expr)),* $(,)?]) => {
			#[test]
			fn $name() {
				let raw = $raw;
				let result: Vec<_> = super::lex(raw, super::Config::default())
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
		do_decompose_starting_delimiter: "zoi fuvi text fu" => [Zoi("zoi"), ZoiDelimiter("fu"), AnyText("vi text "), ZoiDelimiter("fu")],
		dont_decompose_in_quotes: "zoi gy gygy gy" => [Zoi("zoi"), ZoiDelimiter("gy"), AnyText(" gygy "), ZoiDelimiter("gy")],
	}
}
