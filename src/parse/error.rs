use std::borrow::Cow;

use crate::lex::{Selmaho, Token};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("expected token with one of the selmaho {expected:?}, got {got:?}")]
	ExpectedGot {
		expected: Cow<'static, [Selmaho]>,
		got: Option<Token>,
	},
	#[error("nom error: {0:?}")]
	Nom(nom::error::ErrorKind),
	#[error("expected body of zo quote, got EOF")]
	ZoQuoteEof,
}

#[derive(Debug)]
pub struct WithLocation<'a> {
	pub location: &'a [Token],
	pub error: Error,
}

impl<'a> nom::error::ParseError<&'a [Token]> for WithLocation<'a> {
	fn from_error_kind(input: &'a [Token], kind: nom::error::ErrorKind) -> Self {
		Self {
			location: input,
			error: Error::Nom(kind),
		}
	}

	fn append(_input: &'a [Token], _kind: nom::error::ErrorKind, other: Self) -> Self {
		other
	}

	fn from_char(_input: &'a [Token], _ch: char) -> Self {
		unimplemented!("should not occur")
	}

	fn or(self, other: Self) -> Self {
		if self.location.as_ptr() != other.location.as_ptr() {
			return std::cmp::max_by_key(self, other, |error| error.location.as_ptr());
		}

		match (self.error, &other.error) {
			(
				Error::ExpectedGot { mut expected, got },
				Error::ExpectedGot {
					expected: expected_o,
					got: got_o,
				},
			) if &got == got_o => {
				expected.to_mut().extend(expected_o.iter().copied());
				Self {
					location: self.location,
					error: Error::ExpectedGot { expected, got },
				}
			}
			_ => other,
		}
	}
}
