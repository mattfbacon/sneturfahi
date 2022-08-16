// based loosely on https://github.com/lojban/camxes-py/blob/690706f50abf080d746c08da641c11905334298c/camxes_py/parsers/camxes_ilmen.peg

use crate::rules;
use crate::span::Span;

pub fn split_or_trim_condition(ch: char) -> bool {
	".\t\n\r?! ".contains(ch)
}

fn is_consonant(ch: char) -> bool {
	"bcdfgjklmnprstvxz".contains(ch)
}

fn simple_cmevla_check(input: &str) -> bool {
	input
		.chars()
		.rev()
		.filter(|&ch| ch != ',')
		.next()
		.map_or(false, is_consonant)
}

type Input<'input> = impl Iterator<Item = &'input str>;

#[derive(Clone, Copy)]
enum State<'input> {
	Normal,
	Decomposing { rest: &'input str },
}

pub struct Decomposer<'input> {
	input_start: *const u8,
	split: Input<'input>,
	state: State<'input>,
}

#[derive(Clone, Copy)]
enum NextNormalResult<'input> {
	YieldDirectly(Span),
	NeedsDecomposition(&'input str),
}

#[derive(Clone, Copy)]
enum NextDecomposingResult<'input> {
	Continue {
		new_rest: &'input str,
		step_result: Span,
	},
	Break(Span),
	BreakWithNext,
}

impl<'input> Decomposer<'input> {
	fn post_word(input: &str) -> bool {
		rules::nucleus(input).is_none()
			&& (rules::gismu(input).is_some()
				|| rules::fuhivla(input).is_some()
				|| rules::lujvo_minimal(input).is_some()
				|| rules::cmavo_minimal(input).is_some())
	}

	fn next_normal(&self, chunk: &'input str) -> NextNormalResult<'input> {
		log::trace!("chunk of input is {chunk:?}");
		if simple_cmevla_check(chunk) {
			log::trace!("chunk was cmevla, yielding and moving to next chunk");
			NextNormalResult::YieldDirectly(Span::from_embedded_slice(self.input_start, chunk))
		} else {
			log::trace!("chunk was not a cmevla, continuing with decomposition of chunk");
			NextNormalResult::NeedsDecomposition(chunk)
		}
	}

	fn next_decomposing(&self, rest: &'input str) -> NextDecomposingResult<'input> {
		if let Some((cmavo, new_rest)) = rules::cmavo_minimal(rest) {
			log::trace!("considering splitting into ({cmavo:?}, {new_rest:?}), pending post_word check");
			if !new_rest.is_empty() && !new_rest.chars().all(|ch| ch == ',') && Self::post_word(new_rest)
			{
				return NextDecomposingResult::Continue {
					new_rest,
					step_result: Span::from_embedded_slice(self.input_start, cmavo),
				};
			}
		}

		let rest = rest.trim_end_matches(|ch| ch == ',');
		if !rest.is_empty() {
			NextDecomposingResult::Break(Span::from_embedded_slice(self.input_start, rest))
		} else {
			NextDecomposingResult::BreakWithNext
		}
	}
}

impl<'input> Iterator for Decomposer<'input> {
	type Item = Span;

	fn next(&mut self) -> Option<Span> {
		loop {
			match self.state {
				State::Normal => match {
					let chunk = self
						.split
						.by_ref()
						.filter(|chunk| !chunk.is_empty())
						.next()?;
					self.next_normal(chunk)
				} {
					NextNormalResult::YieldDirectly(span) => break Some(span),
					NextNormalResult::NeedsDecomposition(chunk) => {
						self.state = State::Decomposing { rest: chunk };
					}
				},
				State::Decomposing { rest } => match self.next_decomposing(rest) {
					NextDecomposingResult::Continue {
						new_rest,
						step_result,
					} => {
						self.state = State::Decomposing { rest: new_rest };
						break Some(step_result);
					}
					NextDecomposingResult::Break(step_result) => {
						self.state = State::Normal;
						break Some(step_result);
					}
					NextDecomposingResult::BreakWithNext => {
						self.state = State::Normal;
					}
				},
			}
		}
	}
}

impl std::iter::FusedIterator for Decomposer<'_> {}

impl<'input> Decomposer<'input> {
	pub fn next_no_decomposition(&mut self) -> Option<Span> {
		match self.state {
			State::Normal => self.split.next(),
			State::Decomposing { rest } => {
				self.state = State::Normal;
				Some(rest)
			}
		}
		.map(|chunk| Span::from_embedded_slice(self.input_start, chunk))
	}
}

fn _assert_iterator<'input>() {
	fn do_assert<I: Iterator<Item = Span>>() {}
	do_assert::<Decomposer<'input>>();
}

pub fn decompose<'input>(input: &'input str) -> Decomposer<'input> {
	log::debug!("decomposing {input:?}");
	Decomposer {
		input_start: input.as_ptr(),
		split: input.split(split_or_trim_condition),
		state: State::Normal,
	}
}

#[cfg(test)]
mod test {
	macro_rules! make_test {
		($name:ident, $raw:expr, $expected:expr) => {
			#[test]
			fn $name() {
				let raw = $raw;
				let result: Vec<_> = super::decompose(raw)
					.map(|span| span.slice(raw).unwrap())
					.collect();
				assert_eq!(result, &$expected as &[&str]);
			}
		};
	}
	macro_rules! tests {
		($($name:ident : $raw:expr => $expected:expr,)*) => {
			$(make_test!($name, $raw, $expected);)*
		}
	}

	macro_rules! ten_to_the_n_commas {
		(0) => {
			", "
		};
		(1) => {
			concat!(
				ten_to_the_n_commas!(0),
				ten_to_the_n_commas!(0),
				ten_to_the_n_commas!(0),
				ten_to_the_n_commas!(0),
				ten_to_the_n_commas!(0),
				ten_to_the_n_commas!(0),
				ten_to_the_n_commas!(0),
				ten_to_the_n_commas!(0),
				ten_to_the_n_commas!(0),
				ten_to_the_n_commas!(0)
			)
		};
		(2) => {
			concat!(
				ten_to_the_n_commas!(1),
				ten_to_the_n_commas!(1),
				ten_to_the_n_commas!(1),
				ten_to_the_n_commas!(1),
				ten_to_the_n_commas!(1),
				ten_to_the_n_commas!(1),
				ten_to_the_n_commas!(1),
				ten_to_the_n_commas!(1),
				ten_to_the_n_commas!(1),
				ten_to_the_n_commas!(1)
			)
		};
		(3) => {
			concat!(
				ten_to_the_n_commas!(2),
				ten_to_the_n_commas!(2),
				ten_to_the_n_commas!(2),
				ten_to_the_n_commas!(2),
				ten_to_the_n_commas!(2),
				ten_to_the_n_commas!(2),
				ten_to_the_n_commas!(2),
				ten_to_the_n_commas!(2),
				ten_to_the_n_commas!(2),
				ten_to_the_n_commas!(2)
			)
		};
		(4) => {
			concat!(
				ten_to_the_n_commas!(3),
				ten_to_the_n_commas!(3),
				ten_to_the_n_commas!(3),
				ten_to_the_n_commas!(3),
				ten_to_the_n_commas!(3),
				ten_to_the_n_commas!(3),
				ten_to_the_n_commas!(3),
				ten_to_the_n_commas!(3),
				ten_to_the_n_commas!(3),
				ten_to_the_n_commas!(3)
			)
		};
		(5) => {
			concat!(
				ten_to_the_n_commas!(4),
				ten_to_the_n_commas!(4),
				ten_to_the_n_commas!(4),
				ten_to_the_n_commas!(4),
				ten_to_the_n_commas!(4),
				ten_to_the_n_commas!(4),
				ten_to_the_n_commas!(4),
				ten_to_the_n_commas!(4),
				ten_to_the_n_commas!(4),
				ten_to_the_n_commas!(4)
			)
		};
	}

	tests! {
		basic: "gismu" => ["gismu"],
		words: "gismu ko'a jbofuvi" => ["gismu", "ko'a", "jbofuvi"],
		pauses: "gismu.ko'a.jbofuvi" => ["gismu", "ko'a", "jbofuvi"],
		whitespace: "gismu    ko'a     jbofuvi" => ["gismu", "ko'a", "jbofuvi"],
		minajimpe: "minajimpe" => ["mi", "na", "jimpe"],
		tosmabru: "tosmabru" => ["to", "smabru"],
		tosmabru2: "tosymabru" => ["tosymabru"],
		tosmabru3: "totosymabru" => ["to", "tosymabru"],
		how_many: "totototosymabru" => ["to", "to", "to", "tosymabru"],
		fuvi: "fuvi" => ["fu", "vi"],
		sekihu: "seki'u" => ["se", "ki'u"],
		setese: "setese" => ["se", "te", "se"],
		selmaho: "selma'o" => ["selma'o"],
		vowels: "kiiibroda" => ["ki", "ii", "broda"],
		slinkuhi: "loslinku'i" => ["loslinku'i"],
		vowel_prefix: "alobroda" => ["a", "lo", "broda"],
		cmevla_tricky: "alobrodan" => ["alobrodan"],
		cmevla_tricky2: "zo alobrodan alobroda zo" => ["zo", "alobrodan", "a", "lo", "broda", "zo"],
		commas: ",,,m,,,i,,,n,,a,,,j,,,i,,,m,,,p,,,e,,," => [",,,m,,,i", ",,,n,,a", ",,,j,,,i,,,m,,,p,,,e"],
		dont_blow_the_stack: ten_to_the_n_commas!(5) => [],
		srasu: include_str!("srasu.txt") => include!("srasu.txt.expected"),
		vrudysai: "coiiiii" => ["coi", "ii", "ii"],
		janbe: "tanjelavi" => ["tanjelavi"],
		thrig: "mablabigerku" => ["ma", "blabigerku"],
	}
}
