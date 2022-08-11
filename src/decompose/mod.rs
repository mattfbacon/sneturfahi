// based loosely on https://github.com/lojban/camxes-py/blob/690706f50abf080d746c08da641c11905334298c/camxes_py/parsers/camxes_ilmen.peg

fn split_or_trim_condition(ch: char) -> bool {
	".\t\n\r?! ".contains(ch)
}

mod rules;
// mod rules_old;

use rules::ParseResultExt as _;

pub fn decompose<'a>(input: &'a str) -> Vec<&'a str> {
	log::debug!("decomposing {input:?}");

	let mut ret = Vec::new();

	for chunk in input
		.split(split_or_trim_condition)
		.filter(|chunk| !chunk.is_empty())
	{
		log::trace!("chunk of input is {chunk:?}");

		if rules::cmevla(chunk).is_some() {
			log::trace!("chunk was cmevla, not continuing with decomposition");
			ret.push(chunk);
			continue;
		}
		log::trace!("chunk was not a cmevla, continuing with decomposition");

		let mut rest = chunk;
		'cmavo_loop: while let Some((cmavo, new_rest)) = rules::cmavo_not_cvcy_lujvo(rest) {
			log::trace!("considering splitting into ({cmavo:?}, {new_rest:?}), pending post_word check");
			let post_word = new_rest.is_empty()
				|| (rules::nucleus(new_rest).is_none()
					&& ((rules::gismu(new_rest).is_some()
						|| rules::fuhivla(new_rest).is_some()
						|| rules::lujvo(new_rest).is_some())
						|| rules::cmavo_not_cvcy_lujvo(new_rest)
							.and_peek(rules::post_word)
							.is_some()));
			if !post_word {
				log::trace!("splitting into ({cmavo:?}, {new_rest:?}) would leave invalid post_word, so not continuing with decomposition");
				break 'cmavo_loop;
			}

			log::trace!("popped cmavo {cmavo:?} off, leaving {new_rest:?}");
			ret.push(cmavo);
			rest = new_rest;
		}

		if !rest.is_empty() {
			log::trace!("feeding remaining input {rest:?}");
			ret.push(rest);
		}
	}

	ret
}

#[cfg(test)]
mod test {
	macro_rules! make_test {
		($name:ident, $raw:expr, $expected:expr) => {
			#[test]
			fn $name() {
				let result = super::decompose($raw);
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
		fuvi: "fuvi" => ["fu", "vi"],
		sekihu: "seki'u" => ["se", "ki'u"],
		setese: "setese" => ["se", "te", "se"],
		selmaho: "selma'o" => ["selma'o"],
		vowels: "kiiibroda" => ["ki", "ii", "broda"],
		slinkuhi: "loslinku'i" => ["loslinku'i"],
		vowel_prefix: "alobroda" => ["a", "lo", "broda"],
		cmevla_tricky: "alobrodan" => ["alobrodan"],
	}
}
