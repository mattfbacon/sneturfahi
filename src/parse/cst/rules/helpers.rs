use macros::Parse;

use crate::lex::Token;
use crate::parse::cst::error::WithLocation;

pub(super) fn many0<'a, T>(
	parser: impl nom::Parser<&'a [Token], T, WithLocation<'a>>,
) -> impl nom::Parser<&'a [Token], Box<[T]>, WithLocation<'a>> {
	nom::combinator::map(nom::multi::many0(parser), Vec::into_boxed_slice)
}

pub(super) fn many1<'a, T>(
	parser: impl nom::Parser<&'a [Token], T, WithLocation<'a>>,
) -> impl nom::Parser<&'a [Token], Box<[T]>, WithLocation<'a>> {
	nom::combinator::map(nom::multi::many1(parser), Vec::into_boxed_slice)
}

#[derive(Parse)]
pub struct Separated<Item, Separator> {
	pub first: Box<Item>,
	#[parse(with = "many0")]
	pub rest: Box<[(Separator, Item)]>,
}

// print as a single list with the separators interleaved. obviously this would not be valid rust, but it cuts down indentation.
impl<Item: std::fmt::Debug, Separator: std::fmt::Debug> std::fmt::Debug
	for Separated<Item, Separator>
{
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		formatter.write_str("Separated ")?;
		let mut list = formatter.debug_list();
		list.entry(&self.first);
		for (separator, item) in self.rest.iter() {
			list.entry(separator);
			list.entry(item);
		}
		list.finish()?;
		Ok(())
	}
}

#[derive(Debug, Parse)]
pub enum EitherOrBoth<L, R> {
	Right(R),
	Both(L, R),
	Left(L),
}
