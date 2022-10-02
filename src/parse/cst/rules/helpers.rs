use bumpalo::Bump as Arena;
use macros::{Parse, TreeNode};

use crate::lex::Token;
use crate::parse::cst::error::WithLocation;
use crate::parse::tree_node::{TreeNode, TreeNodeChild};

pub(super) fn many0<'a, 'arena, T: 'arena>(
	parser: impl nom::Parser<&'a [Token], T, WithLocation<'a>>,
	arena: &'arena Arena,
) -> impl nom::Parser<&'a [Token], &'arena [T], WithLocation<'a>> {
	nom::combinator::map(nom::multi::many0(parser), |parsed| {
		&*arena.alloc_slice_fill_iter(parsed.into_iter())
	})
}

pub(super) fn many1<'a, 'arena, T: 'arena>(
	parser: impl nom::Parser<&'a [Token], T, WithLocation<'a>>,
	arena: &'arena Arena,
) -> impl nom::Parser<&'a [Token], &'arena [T], WithLocation<'a>> {
	nom::combinator::map(nom::multi::many1(parser), |parsed| {
		&*arena.alloc_slice_fill_iter(parsed.into_iter())
	})
}

pub(super) struct PassArena<'arena, T>(&'arena Arena, std::marker::PhantomData<fn() -> T>);

impl<'arena, T> PassArena<'arena, T> {
	pub(super) fn new(arena: &'arena Arena) -> Self {
		Self(arena, std::marker::PhantomData)
	}
}

impl<'a: 'arena, 'arena, T: super::super::parse_trait::Parse<'arena>>
	nom::Parser<&'a [Token], T, WithLocation<'a>> for PassArena<'arena, T>
{
	fn parse(&mut self, input: &'a [Token]) -> super::ParseResult<'a, T> {
		T::parse(input, self.0)
	}
}

#[derive(Parse)]
pub struct Separated<'arena, Item, Separator> {
	pub first: &'arena Item,
	#[parse(with = "many0")]
	pub rest: &'arena [(Separator, Item)],
}

impl<Item: TreeNodeChild, Separator: TreeNodeChild> TreeNode for Separated<'_, Item, Separator> {
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
			.find_map(|(separator, item)| item.end_location().or_else(|| separator.end_location()))
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
	for Separated<'_, Item, Separator>
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
