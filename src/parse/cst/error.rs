//! Parsing errors.
//!
//! This module centers around [`Error`], the type for errors that can occur while parsing, and [`WithLocation`], a wrapper type that adds a location to the error.

use std::borrow::Cow;

use crate::lex::{Selmaho, Token};

/// Errors that can occur while parsing.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
	/// The parser expected one of the selmaho in `expected`, but got the token in `got`.
	#[error("expected token with one of the selmaho {expected:?}, got {got:?}")]
	ExpectedGot {
		/// The tokens that were expected.
		expected: Cow<'static, [Selmaho]>,
		/// The token that was found, or `None` for EOF.
		got: Option<Token>,
	},
	/// A parsing post-condition failed.
	#[error("post-condition failed: {0:?}")]
	PostConditionFailed(&'static str),
	/// Forwards errors from nom combinators.
	#[error("nom error: {0:?}")]
	Nom(nom::error::ErrorKind),
	/// The parser expected the body of a zo quote (a word) but found EOF.
	///
	/// This is similar to `ExpectedGot`, but zo accepts almost any selmaho to be quoted so `expected` would be unreasonably large.
	#[error("expected body of zo quote, got EOF")]
	ZoQuoteEof,
	/// Sometimes we require rules to consume input to avoid them "succeeding" but really matching nothing.
	#[error("{0} consumed no input but was expected to")]
	Empty(&'static str),
}

impl Error {
	/// Convert this error to a [`WithLocation`] by providing a location.
	#[must_use]
	pub fn with_location(self, location: &[Token]) -> WithLocation<'_> {
		WithLocation {
			location,
			error: self,
		}
	}
}

/// An error with an embedded location, indicating where the error occurred.
#[derive(Debug)]
pub struct WithLocation<'a> {
	/// The location where the error occurred, stored as the tokens that were remaining when the error occurred.
	pub location: &'a [Token],
	/// The error itself.
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
