//! Decomposition of Lojban text into morphemes.
//!
//! This module centers around the [decompose] function, which is heavily documented.

// based loosely on https://github.com/lojban/camxes-py/blob/690706f50abf080d746c08da641c11905334298c/camxes_py/parsers/camxes_ilmen.peg

use crate::rules;
use crate::span::{Location, Span};

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
	"bcdfgjklmnprstvxz".contains(ch.to_ascii_lowercase())
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

#[must_use]
fn post_word(input: &str) -> bool {
	rules::nucleus(input).is_none()
		&& (rules::gismu(input).is_some()
			|| rules::fuhivla(input).is_some()
			|| rules::lujvo_minimal(input).is_some()
			|| rules::cmavo_minimal(input).is_some())
}

fn decompose_single(input: &str) -> Vec<Span> {
	let to_span = |slice| Span::from_embedded_slice(input.as_ptr(), slice);

	if simple_cmevla_check(input) {
		return vec![to_span(input)];
	}

	let mut rest = input;
	let mut chunks = Vec::new();
	let mut saved_decomposed = None;

	// having *another* cmavo fall off means that the first one is definitely valid, even without a `post_word` check.
	// this is more efficient than a `post_word` check so we use it as our primary mechanism.
	while let Some((fallen_off, new_rest)) =
		rules::cmavo_minimal(rest).or_else(|| rules::explicitly_stressed_brivla_minimal(rest))
	{
		if let Some((saved_decomposed, _)) = saved_decomposed.replace((fallen_off, rest)) {
			chunks.push(to_span(saved_decomposed));
		}
		rest = new_rest;
	}

	// but of course, we might need at least one `post_word` check.
	if let Some((saved_decomposed, old_rest)) = saved_decomposed {
		if post_word(rest) {
			chunks.push(to_span(saved_decomposed));
		} else {
			rest = old_rest;
		}
	}

	rest = rest.trim_end_matches(',');
	if !rest.is_empty() {
		chunks.push(to_span(rest));
	}

	chunks
}

type Chunks<'input> = std::iter::Peekable<std::str::Split<'input, fn(char) -> bool>>;

/// The iterator used for decomposition.
///
/// The public way to create an instance of this type is [`decompose`], and the documentation for decomposition in general is there.
#[derive(Debug, Clone)]
pub struct Decomposer<'input> {
	chunks: Chunks<'input>,
	current_chunk: std::vec::IntoIter<Span>,
	input: &'input str,
}

impl<'input> Iterator for Decomposer<'input> {
	type Item = Span;

	fn next(&mut self) -> Option<Span> {
		loop {
			// possibly exit loop with `Some`
			if let Some(word) = self.current_chunk.next() {
				return Some(word);
			}

			// possibly exit loop with `None` (via `?`)
			let next_chunk = self
				.chunks
				.find(|chunk| !chunk.is_empty() && !chunk.bytes().all(|ch| ch == b','))?;
			let offset =
				Location::try_from(next_chunk.as_ptr() as usize - self.input.as_ptr() as usize).unwrap();
			let mut chunk_words = decompose_single(next_chunk);

			for word in &mut chunk_words {
				word.start += offset;
				word.end += offset;
			}

			self.current_chunk = chunk_words.into_iter();
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let lower = self
			.current_chunk
			.as_slice()
			.len()
			.saturating_add(self.chunks.size_hint().0);
		(lower, None)
	}
}

impl<'input> Decomposer<'input> {
	fn new(input: &'input str) -> Self {
		Self {
			chunks: input
				.split(split_or_trim_condition as fn(char) -> bool)
				.peekable(),
			current_chunk: vec![].into_iter(), // dummy
			input,
		}
	}

	fn slice_to_span(&self, slice: &str) -> Span {
		Span::from_embedded_slice(self.input.as_ptr(), slice)
	}

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
	#[allow(clippy::missing_panics_doc)] // doesn't actually panic in practice
	pub fn next_no_decomposition(&mut self) -> Option<Span> {
		if let Some(remaining_decomposed) = self.current_chunk.next() {
			let start = remaining_decomposed.start;
			let end = self.chunks.peek().copied().map_or_else(
				|| self.input.len().try_into().unwrap(),
				|chunk| self.slice_to_span(chunk).start - 1,
			);
			self.current_chunk = vec![].into_iter();
			Some(Span { start, end })
		} else {
			self.chunks.next().map(|chunk| self.slice_to_span(chunk))
		}
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
	Decomposer::new(input)
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
		stress1_baseline: "lojboprenu" => ["lo", "jboprenu"],
		stress1_1: "LOjboPREnu" => ["LOjbo", "PREnu"],
		stress1_2: "lojboPREnu" => ["lo", "jboPREnu"],
		stress2_baseline: "mipramido" => ["mi", "pramido"],
		stress2_1: "miPRAmido" => ["mi", "PRAmi", "do"],
		stress2_2: "MIpramido" => ["MIpra", "mi", "do"],
		numbers: "li123" => ["li", "1", "2", "3"],
		numbers1: "li 123" => ["li", "1", "2", "3"],
		numbers2: "123moi" => ["1", "2", "3", "moi"],
		yyy: "yyy" => ["yyy"],
		yyy2: "mi yyy broda" => ["mi", "yyy", "broda"],
		yyy3: "mi yyybroda" => ["mi", "yyy", "broda"],
	}
}
