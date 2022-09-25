use crate::lex::Token;

pub mod error;
mod parse_trait;
pub mod rules;

pub use error::Error;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Cst {
	root: rules::Root,
}

impl Cst {
	/// Parse tokens into a concrete syntax tree.
	#[allow(clippy::missing_errors_doc)] // obvious
	pub fn parse(input: &[Token]) -> Result<Self, error::WithLocation<'_>> {
		nom::Finish::finish(nom::combinator::all_consuming(
			<rules::Root as parse_trait::Parse>::parse,
		)(input))
		.map(|(rest, root)| {
			debug_assert!(rest.is_empty());
			root
		})
		.map(|root| Self { root })
	}

	/// Get the root of the CST, which allows traversing the entire CST.
	#[must_use]
	pub fn root(&self) -> &rules::Root {
		&self.root
	}
}
