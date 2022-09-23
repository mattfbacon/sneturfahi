use nom::combinator::{all_consuming, map};
use nom::sequence::tuple;
use nom::Parser;

use crate::lex::Token;

pub mod cst;
pub mod error;
#[cfg(test)]
mod tests;

use cst::Parse;
pub use error::Error;

type ParseResult<'a, T> = nom::IResult<&'a [Token], T, error::WithLocation<'a>>;

fn many0<'a, T>(
	parser: impl Parser<&'a [Token], T, error::WithLocation<'a>>,
) -> impl Parser<&'a [Token], Box<[T]>, error::WithLocation<'a>> {
	map(nom::multi::many0(parser), Vec::into_boxed_slice)
}

fn many1<'a, T>(
	parser: impl Parser<&'a [Token], T, error::WithLocation<'a>>,
) -> impl Parser<&'a [Token], Box<[T]>, error::WithLocation<'a>> {
	map(nom::multi::many1(parser), Vec::into_boxed_slice)
}

fn selmaho_raw<T: cst::SelmahoTypeRaw>(input: &[Token]) -> ParseResult<'_, T> {
	let mut input = input.iter();
	T::try_from(input.next().copied())
		.map(|matched| (input.as_slice(), matched))
		.map_err(|error| {
			nom::Err::Error(error::WithLocation {
				location: input.as_slice(),
				error,
			})
		})
}

#[derive(Debug)]
pub struct Cst {
	root: cst::Root,
}

impl Cst {
	/// Parse tokens into a concrete syntax tree.
	#[allow(clippy::missing_errors_doc)] // obvious
	pub fn parse(input: &[Token]) -> Result<Self, error::WithLocation<'_>> {
		nom::Finish::finish(all_consuming(cst::Text::parse)(input))
			.map(|(rest, root)| {
				debug_assert!(rest.is_empty());
				root
			})
			.map(|root| Self { root })
	}

	/// Get the root of the CST, which allows traversing the entire CST.
	#[must_use]
	pub fn root(&self) -> &cst::Root {
		&self.root
	}
}
