use nom::combinator::{all_consuming, cut, map};
use nom::sequence::tuple;
use nom::Parser;

use crate::lex::Token;

pub mod cst;
pub mod error;
#[cfg(test)]
mod tests;

use cst::Parse;
pub use error::Error;

pub type ParseResult<'a, T> = nom::IResult<&'a [Token], T, error::WithLocation<'a>>;

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

/// Whether to set `should_cut` is a bit of a tricky question.
/// It is complicated by the elision of elidable terminators.
/// For example, `lo broda joi lo brode` is perfectly acceptable and implies a `KU` before `joi`.
/// However, if `should_cut` was set to true, the failure to parse a selbri after the `joi` (since selbri can also be connected with `JOI`) would have caused a parse *failure* (not an error).
/// Thus, in that situation `should_cut` must be false.
fn separated<'a, Item: Parse, Separator: Parse>(
	should_cut: bool,
) -> impl Parser<&'a [Token], cst::Separated<Item, Separator>, error::WithLocation<'a>> + Clone {
	move |input| {
		map(
			tuple((
				<Box<Item>>::parse,
				many0(tuple((Separator::parse, |input| {
					if should_cut {
						cut(Item::parse)(input)
					} else {
						Item::parse(input)
					}
				}))),
			)),
			|(first, rest)| cst::Separated { first, rest },
		)(input)
	}
}

/// Parse tokens into a concrete syntax tree.
#[allow(clippy::missing_errors_doc)] // obvious
pub fn parse(input: &[Token]) -> Result<cst::Root, error::WithLocation<'_>> {
	nom::Finish::finish(all_consuming(cst::Text::parse)(input)).map(|(rest, root)| {
		debug_assert!(rest.is_empty());
		root
	})
}
