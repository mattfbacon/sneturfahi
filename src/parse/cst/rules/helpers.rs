use macros::{Parse, TreeNode};

use crate::lex::Token;
use crate::parse::cst::error::WithLocation;
use crate::parse::tree_node::{TreeNode, TreeNodeChild};

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

impl<Item: TreeNodeChild, Separator: TreeNodeChild> TreeNode for Separated<Item, Separator> {
	fn name(&self) -> &'static str {
		"Separated"
	}

	fn experimental(&self) -> bool {
		self.first.experimental()
			|| self
				.rest
				.iter()
				.any(|(separator, item)| separator.experimental() || item.experimental())
	}

	fn start_location(&self) -> Option<crate::span::Location> {
		self.first.start_location().or_else(|| {
			self
				.rest
				.iter()
				.find_map(|(separator, item)| separator.start_location().or_else(|| item.start_location()))
		})
	}

	fn end_location(&self) -> Option<crate::span::Location> {
		self
			.rest
			.iter()
			.rev()
			.find_map(|(separator, item)| separator.start_location().or_else(|| item.end_location()))
			.or_else(|| self.first.end_location())
	}

	fn for_each_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn TreeNode)) {
		self.first.invoke_with_self(f);
		for (separator, item) in self.rest.iter() {
			separator.invoke_with_self(f);
			item.invoke_with_self(f);
		}
	}
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

#[derive(Debug, Parse, TreeNode)]
#[tree_node(passthrough_child)]
pub enum EitherOrBoth<L, R> {
	Right(R),
	Both(L, R),
	Left(L),
}
