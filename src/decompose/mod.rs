//! Decomposition of Lojban text into morphemes.
//!
//! This module centers around the [decompose] function, which is heavily documented.

// based loosely on https://github.com/lojban/camxes-py/blob/690706f50abf080d746c08da641c11905334298c/camxes_py/parsers/camxes_ilmen.peg

use crate::rules;
use crate::span::Span;

/// The condition to be used for splitting and/or trimming Lojban text in general.
///
/// It matches whitespace and pauses, as well as a couple of other "pause-like" characters.
/// Note that this function does not perform any kind of decomposition, only naive splitting. Use [`decompose`] if you need full decomposition.
///
/// # Examples
///
/// Splitting:
///
/// ```rust
/// # use sneturfahi::decompose::split_or_trim_condition;
/// let text_with_pauses = "a.b.c";
/// let split: Vec<_> = text_with_pauses.split(split_or_trim_condition).collect();
/// assert_eq!(split, ["a", "b", "c"]);
/// ```
///
/// Trimming:
///
/// ```rust
/// # use sneturfahi::decompose::split_or_trim_condition;
/// let text_surrounded_by_junk = ". text .";
/// let trimmed = text_surrounded_by_junk.trim_matches(split_or_trim_condition);
/// assert_eq!(trimmed, "text");
/// ```
#[must_use]
pub fn split_or_trim_condition(ch: char) -> bool {
	".\t\n\r?! ".contains(ch)
}

#[must_use]
fn is_consonant(ch: char) -> bool {
	"bcdfgjklmnprstvxz".contains(ch)
}

// while the input should never be empty, it may be only commas.
// in that case calling `next` on the iterator would in fact yield `None`, so we need to handle that case with `map_or` rather than `unwrap`ping.
#[must_use]
fn simple_cmevla_check(input: &str) -> bool {
	input
		.chars()
		.rev()
		.find(|&ch| ch != ',')
		.map_or(false, is_consonant)
}

type Input<'input> = impl Iterator<Item = &'input str>;

#[derive(Clone, Copy)]
enum State<'input> {
	Normal,
	Decomposing { rest: &'input str },
}

/// The iterator used for decomposition.
///
/// The public way to create an instance of this type is [`decompose`], and the documentation for decomposition in general is there.
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
	#[must_use]
	fn post_word(input: &str) -> bool {
		rules::nucleus(input).is_none()
			&& (rules::gismu(input).is_some()
				|| rules::fuhivla(input).is_some()
				|| rules::lujvo_minimal(input).is_some()
				|| rules::cmavo_minimal(input).is_some())
	}

	#[must_use]
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

	#[must_use]
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
		if rest.is_empty() {
			NextDecomposingResult::BreakWithNext
		} else {
			NextDecomposingResult::Break(Span::from_embedded_slice(self.input_start, rest))
		}
	}
}

impl<'input> Iterator for Decomposer<'input> {
	type Item = Span;

	fn next(&mut self) -> Option<Span> {
		loop {
			match self.state {
				State::Normal => match {
					let chunk = self.split.find(|chunk| !chunk.is_empty())?;
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
	/// Get the next token without performing any decomposition.
	///
	/// It acts similarly to splitting with [`split_or_trim_condition`] but maintains the correct state of the decomposer so that normal, decomposing iteration can resume after using this function.
	///
	/// # Examples
	///
	/// ```rust
	/// # use sneturfahi::decompose::decompose;
	/// let input = "minajimpe donajimpe ko'anajimpe";
	/// let mut decomposer = decompose(input);
	/// assert_eq!(decomposer.next().unwrap().slice(input).unwrap(), "mi");
	/// assert_eq!(
	/// 	decomposer
	/// 		.next_no_decomposition()
	/// 		.unwrap()
	/// 		.slice(input)
	/// 		.unwrap(),
	/// 	"najimpe"
	/// );
	/// assert_eq!(
	/// 	decomposer
	/// 		.next_no_decomposition()
	/// 		.unwrap()
	/// 		.slice(input)
	/// 		.unwrap(),
	/// 	"donajimpe"
	/// );
	/// assert_eq!(
	/// 	decomposer
	/// 		.map(|span| span.slice(input).unwrap())
	/// 		.collect::<Vec<_>>(),
	/// 	["ko'a", "na", "jimpe"]
	/// );
	/// ```
	///
	/// This function also doesn't filter empty chunks like plain `next` does:
	///
	/// ```rust
	/// # use sneturfahi::decompose::decompose;
	/// let input = "zoi gy   gy"; // an example of a context where empty chunks might be important
	/// let mut decomposer = decompose(input);
	/// let result = std::iter::from_fn(|| decomposer.next_no_decomposition())
	/// 	.map(|span| span.slice(input).unwrap())
	/// 	.collect::<Vec<_>>();
	/// assert_eq!(result, ["zoi", "gy", "", "", "gy"]);
	/// ```
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

fn _assert_iterator() {
	fn do_assert<I: Iterator<Item = Span>>() {}
	do_assert::<Decomposer<'_>>();
}

/// Create a [`Decomposer`] to decompose the Lojban text `input` into morphemes.
///
/// This does not do any classification of the decomposed morphemes; for that, use the utilities provided by the [`lex`] module.
/// Note that the decomposer yields spans. For more information about spans including how to resolve them to text, see the [`span`] module.
///
/// # Examples
///
/// To start, this function provides a superset of the functionality of splitting by `split_or_trim_condition`. It splits on the same conditions:
///
/// ```rust
/// # use sneturfahi::decompose::decompose;
/// let input = "broda broda.broda!broda";
/// let decomposed: Vec<_> = decompose(input)
/// 	.map(|span| span.slice(input).unwrap())
/// 	.collect();
/// assert_eq!(decomposed, ["broda", "broda", "broda", "broda"]);
/// ```
///
/// However, this function also decomposes compounds like "seki'u", "tosmabru", "minajimpe", and "alobroda":
///
/// ```rust
/// # use sneturfahi::decompose::decompose;
/// let examples = [
/// 	("seki'u", &["se", "ki'u"] as &[&str]),
/// 	("tosmabru", &["to", "smabru"]),
/// 	("minajimpe", &["mi", "na", "jimpe"]),
/// 	("alobroda", &["a", "lo", "broda"]),
/// ];
/// for (input, expected) in examples {
/// 	let decomposed: Vec<_> = decompose(input)
/// 		.map(|span| span.slice(input).unwrap())
/// 		.collect();
/// 	assert_eq!(decomposed, expected);
/// }
/// ```
///
/// It also recognizes cmevla:
///
/// ```rust
/// # use sneturfahi::decompose::decompose;
/// // "alobroda" would decompose as shown above
/// let input = "alobrodan";
/// let decomposed: Vec<_> = decompose(input)
/// 	.map(|span| span.slice(input).unwrap())
/// 	.collect();
/// assert_eq!(decomposed, ["alobrodan"]);
/// ```
///
/// And fuhivla:
///
/// ```rust
/// # use sneturfahi::decompose::decompose;
/// // "blabigerku" may look like two gismu but is actually one fuhivla
/// let input = "mablabigerku";
/// let decomposed: Vec<_> = decompose(input)
/// 	.map(|span| span.slice(input).unwrap())
/// 	.collect();
/// assert_eq!(decomposed, ["ma", "blabigerku"]);
/// ```
///
/// When iterating normally, the decomposer will not yield empty chunks:
///
/// ```rust
/// # use sneturfahi::decompose::decompose;
/// let input = "    mi     nelci        lo        canlu   ";
/// assert!(decompose(input).all(|span| !span.is_empty()));
/// ```
///
/// Note the [`Decomposer::next_no_decomposition`] method, which can yield empty chunks.
///
/// [lex]: mod@crate::lex
/// [span]: crate::span
#[must_use]
pub fn decompose(input: &str) -> Decomposer<'_> {
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
		srasu: include_str!("../srasu.txt") => include!("srasu.txt.expected"),
		vrudysai: "coiiiii" => ["coi", "ii", "ii"],
		janbe: "tanjelavi" => ["tanjelavi"],
		thrig: "mablabigerku" => ["ma", "blabigerku"],
	}
}
