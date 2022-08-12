// based loosely on https://github.com/lojban/camxes-py/blob/690706f50abf080d746c08da641c11905334298c/camxes_py/parsers/camxes_ilmen.peg

use crate::span::Span;

fn split_or_trim_condition(ch: char) -> bool {
	".\t\n\r?! ".contains(ch)
}

mod rules;

fn is_consonant(ch: char) -> bool {
	"bcdfgjklmnprstvxz".contains(ch)
}

fn simple_cmevla_check(input: &str) -> bool {
	is_consonant(input.chars().rev().filter(|&ch| ch != ',').next().unwrap())
}

pub fn decompose<'a>(input: &'a str) -> impl Iterator<Item = Span<'a>> {
	let generator = move || {
		log::debug!("decomposing {input:?}");

		'chunks: for chunk in input
			.split(split_or_trim_condition)
			.filter(|chunk| !chunk.is_empty())
		{
			log::trace!("chunk of input is {chunk:?}");

			if simple_cmevla_check(input) {
				log::trace!("chunk was cmevla, not continuing with decomposition");
				yield Span::from_slice(input);
				continue 'chunks;
			}
			log::trace!("chunk was not a cmevla, continuing with decomposition");

			fn post_word(input: &str) -> bool {
				rules::nucleus(input).is_none()
					&& (rules::gismu(input).is_some()
						|| rules::fuhivla(input).is_some()
						|| rules::lujvo_minimal(input).is_some()
						|| rules::cmavo_minimal(input).is_some())
			}

			let mut rest = chunk;
			'cmavo: while let Some((cmavo, new_rest)) = rules::cmavo_minimal(rest) {
				log::trace!(
					"considering splitting into ({cmavo:?}, {new_rest:?}), pending post_word check"
				);
				if new_rest.is_empty() || new_rest.chars().all(|ch| ch == ',') {
					break 'cmavo;
				}

				if !post_word(new_rest) {
					log::trace!("splitting into ({cmavo:?}, {new_rest:?}) would leave invalid post_word, so not continuing with decomposition");
					break 'cmavo;
				}

				log::trace!("popped cmavo {cmavo:?} off, leaving {new_rest:?}");
				yield Span::from_embedded_slice(input.as_ptr(), cmavo);
				rest = new_rest;
			}

			rest = rest.trim_end_matches(|ch| ch == ',');
			if !rest.is_empty() {
				log::trace!("feeding remaining input {rest:?}");
				yield Span::from_embedded_slice(input.as_ptr(), rest);
			}
		}
	};
	std::iter::from_generator(generator)
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

	tests! {
		basic: "gismu" => ["gismu"],
		words: "gismu ko'a jbofuvi" => ["gismu", "ko'a", "jbofuvi"],
		pauses: "gismu.ko'a.jbofuvi" => ["gismu", "ko'a", "jbofuvi"],
		whitespace: "gismu    ko'a     jbofuvi" => ["gismu", "ko'a", "jbofuvi"],
		minajimpe: "minajimpe" => ["mi", "na", "jimpe"],
		tosmabru: "tosmabru" => ["to", "smabru"],
		tosmabru2: "tosymabru" => ["tosymabru"],
		tosmabru3: "totosymabru" => ["to", "tosymabru"],
		how_many: "totototosymabru" => ["to", "to", "to", "tosymabru"], // this test is very slow
		fuvi: "fuvi" => ["fu", "vi"],
		sekihu: "seki'u" => ["se", "ki'u"],
		setese: "setese" => ["se", "te", "se"],
		selmaho: "selma'o" => ["selma'o"],
		vowels: "kiiibroda" => ["ki", "ii", "broda"],
		slinkuhi: "loslinku'i" => ["loslinku'i"],
		vowel_prefix: "alobroda" => ["a", "lo", "broda"],
		cmevla_tricky: "alobrodan" => ["alobrodan"],
		commas: ",,,m,,,i,,,n,,a,,,j,,,i,,,m,,,p,,,e,,," => [",,,m,,,i", ",,,n,,a", ",,,j,,,i,,,m,,,p,,,e"],
		srasu: include_str!("srasu.txt") => include!("srasu.txt.expected"),
		vrudysai: "coiiiii" => ["coi", "ii", "ii"],
		janbe: "tanjelavi" => ["tanjelavi"],
	}
}
