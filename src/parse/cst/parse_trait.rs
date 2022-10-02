use bumpalo::Bump as Arena;

use crate::lex::Token;
use crate::parse::cst::error::{Error, WithLocation};

pub(in crate::parse::cst) type Result<'a, T> = nom::IResult<&'a [Token], T, WithLocation<'a>>;

pub(in crate::parse::cst) trait Parse<'arena>: Sized {
	fn parse<'a: 'arena>(input: &'a [Token], arena: &'arena Arena) -> Result<'a, Self>;
}

impl<'arena, T: Parse<'arena>> Parse<'arena> for Option<T> {
	fn parse<'a: 'arena>(input: &'a [Token], arena: &'arena Arena) -> Result<'a, Self> {
		nom::combinator::opt(|input| Parse::parse(input, arena))(input)
	}
}

impl<'arena, T: Parse<'arena>> Parse<'arena> for &'arena T {
	fn parse<'a: 'arena>(input: &'a [Token], arena: &'arena Arena) -> Result<'a, Self> {
		nom::combinator::map(
			|input| Parse::parse(input, arena),
			|parsed| &*arena.alloc(parsed),
		)(input)
	}
}

impl<'arena> Parse<'arena> for Token {
	fn parse<'a: 'arena>(input: &'a [Token], _: &'arena Arena) -> Result<'a, Self> {
		let mut input = input.iter();
		input
			.next()
			.map(|&token| (input.as_slice(), token))
			.ok_or(nom::Err::Error(WithLocation {
				location: input.as_slice(),
				error: Error::Nom(nom::error::ErrorKind::Eof),
			}))
	}
}

impl<'arena> Parse<'arena> for crate::Span {
	fn parse<'a: 'arena>(input: &'a [Token], arena: &'arena Arena) -> Result<'a, Self> {
		Token::parse(input, arena).map(|(rest, matched)| (rest, matched.span))
	}
}

macro_rules! tuple_impls {
	// base case
	() => {};
	(@single $($idents:ident),*) => {
		impl<'arena, $($idents: Parse<'arena>),*> Parse<'arena> for ($($idents,)*) {
			fn parse<'a: 'arena>(input: &'a [Token], arena: &'arena Arena) -> Result<'a, Self> {
				nom::sequence::tuple(($(|input| <$idents as Parse>::parse(input, arena),)*))(input)
			}
		}
	};
	($first:ident $(, $idents:ident)*) => {
		tuple_impls!(@single $first $(, $idents)*);
		tuple_impls!($($idents),*);
	};
}

tuple_impls![T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15];
