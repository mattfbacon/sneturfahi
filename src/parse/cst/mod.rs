use crate::lex::Token;
use crate::parse::Arena;

pub mod error;
mod parse_trait;
pub mod rules;

pub use error::Error;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Cst<'arena> {
	root: &'arena rules::Root<'arena>,
}

impl<'arena> Cst<'arena> {
	/// Parse tokens into a concrete syntax tree.
	#[allow(clippy::missing_errors_doc)] // obvious
	pub fn parse<'a: 'arena>(
		input: &'a [Token],
		arena: &'arena Arena,
	) -> Result<Self, error::WithLocation<'a>> {
		let parsed = nom::Finish::finish(nom::combinator::all_consuming(|input| {
			<rules::Root<'arena> as parse_trait::Parse>::parse(input, &arena.0)
		})(input))
		.map(|(rest, root)| {
			debug_assert!(rest.is_empty());
			root
		});
		parsed.map(|root| Self {
			root: arena.0.alloc(root),
		})
	}

	/// Get the root of the CST, which allows traversing the entire CST.
	#[must_use]
	pub fn root(&self) -> &'arena rules::Root<'arena> {
		self.root
	}
}
