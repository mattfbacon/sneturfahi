use crate::lex::{self, Token};

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug)]
pub struct Ast<'a> {
	_dummy: &'a str,
}

pub fn parse<'a>(_input: impl Iterator<Item = Result<Token, lex::Error>>) -> Result<Ast<'a>> {
	todo!()
}
