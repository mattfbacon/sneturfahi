#[derive(Debug)]
pub enum Token<'a> {
	Dummy(&'a str),
}

pub fn lex(_input: &str) -> Vec<Token<'_>> {
	todo!()
}
