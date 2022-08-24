use crate::lex::{Selmaho, Token};

pub(in crate::parse) struct Cursor<'a> {
	pub(in crate::parse) input: &'a [Token],
	pub(in crate::parse) cursor: usize,
}

impl Cursor<'_> {
	pub(in crate::parse) fn next(&mut self) -> Option<Token> {
		let &value = self.input.get(self.cursor)?;
		self.cursor += 1;
		Some(value)
	}

	pub(in crate::parse) fn peek(&self) -> Option<Token> {
		self.input.get(self.cursor).copied()
	}

	pub(in crate::parse) fn peek_selmaho(&self) -> Option<Selmaho> {
		self.peek().map(|token| token.selmaho)
	}

	pub(in crate::parse) fn next_if(
		&mut self,
		condition: impl FnOnce(Token) -> bool,
	) -> Option<Token> {
		let &value = self.input.get(self.cursor)?;
		if condition(value) {
			self.cursor += 1;
			Some(value)
		} else {
			None
		}
	}

	pub(in crate::parse) fn next_if_selmaho(&mut self, selmaho: Selmaho) -> Option<Token> {
		self.next_if(move |token| token.selmaho == selmaho)
	}
}
