use crate::lex::Token;
use crate::parse::cst::error::{Error, WithLocation};

pub(in crate::parse::cst) type Result<'a, T> = nom::IResult<&'a [Token], T, WithLocation<'a>>;

pub(in crate::parse::cst) trait Parse: Sized {
	fn parse(input: &[Token]) -> Result<'_, Self>;
}

impl<T: Parse> Parse for Option<T> {
	fn parse(input: &[Token]) -> Result<'_, Self> {
		nom::combinator::opt(Parse::parse)(input)
	}
}

impl<T: Parse> Parse for Box<T> {
	fn parse(input: &[Token]) -> Result<'_, Self> {
		nom::combinator::map(Parse::parse, Box::new)(input)
	}
}

impl Parse for Token {
	fn parse(input: &[Token]) -> Result<'_, Self> {
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

impl Parse for crate::Span {
	fn parse(input: &[Token]) -> Result<'_, Self> {
		Token::parse(input).map(|(rest, matched)| (rest, matched.span))
	}
}

macro_rules! tuple_impls {
	// base case
	() => {};
	(@single $($idents:ident),*) => {
		impl<$($idents: Parse),*> Parse for ($($idents,)*) {
			fn parse(input: &[Token]) -> Result<'_, Self> {
				nom::sequence::tuple(($(<$idents as Parse>::parse,)*))(input)
			}
		}
	};
	($first:ident $(, $idents:ident)*) => {
		tuple_impls!(@single $first $(, $idents)*);
		tuple_impls!($($idents),*);
	};
}

tuple_impls![T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15];
