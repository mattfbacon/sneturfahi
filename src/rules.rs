#![allow(dead_code, unused_macros)]
#![allow(clippy::unnecessary_wraps)] // consistency

use std::cell::Cell;

use macros::debug_rule;

std::thread_local! {
	pub static PARSER_LEVEL: Cell<usize> = Cell::new(0);
}

macro_rules! debug_rule_start {
	($name:expr) => {
		PARSER_LEVEL.with(|stack| {
			let name = $name;
			let level = stack.replace(stack.get() + 1);
			log::trace!("{:width$ }entering {name:?} rule", "", width = level * 2);
		})
	};
}

macro_rules! debug_rule_end {
	($name:expr, $result:expr) => {
		PARSER_LEVEL.with(|stack| {
			let name = $name;
			let level = stack.replace(stack.get().checked_sub(1).unwrap()) - 1;
			let message = if $result.is_some() {
				"succeeded"
			} else {
				"failed"
			};
			log::trace!(
				"{:width$ }exiting {name:?} rule: {message}",
				"",
				width = level * 2
			);
		})
	};
}

macro_rules! impl_parse {
	() => {
		impl (for<'impl_parse> Fn(&'impl_parse str) -> Option<(&'impl_parse str, &'impl_parse str)>)
	}
}

macro_rules! not {
	($val:expr) => {
		if $val.is_some() {
			return None;
		}
	};
}

fn null(input: &str) -> ParseResult<'_> {
	Some(input.split_at(0))
}

fn not(parser: impl_parse!()) -> impl_parse!() {
	move |input| {
		if parser(input).is_some() {
			None
		} else {
			Some(input.split_at(0))
		}
	}
}

macro_rules! peek {
	($val:expr) => {
		if $val.is_none() {
			return None;
		}
	};
}

fn peek(parser: impl_parse!()) -> impl_parse!() {
	move |input| parser(input).and(null(input))
}

macro_rules! or {
	($($parsers:expr),+ $(,)?) => {
		move |input| {
			$(if let Some(ret) = $parsers(input) { return Some(ret); })+
			return None;
		}
	};
	(longest=> $($parsers:expr),+ $(,)?) => {
		move |input| {
			let results = [$($parsers(input),)+];
			results.into_iter().filter_map(|result| result).min_by_key(|(_matched, rest)| rest.len())
		}
	};
}

fn opt(parser: impl_parse!()) -> impl_parse!() {
	or![parser, null]
}

macro_rules! seq {
	($first:expr $(, $parsers:expr)* $(,)?) => {
		|input| {
			let (first_matched, mut rest) = $first(input)?;
			let first_ptr = first_matched.as_ptr() as usize;
			#[allow(unused_assignments)] // the expanded other parsers could overwrite them
			let mut last_ptr = first_ptr + first_matched.len();
			$({
				let (matched, new_rest) = $parsers(rest)?;
				#[allow(unused_assignments)] // the expanded other parsers could overwrite them
				{ last_ptr = matched.as_ptr() as usize + matched.len(); }
				rest = new_rest;
			})*
			let input_ptr = input.as_ptr() as usize;
				Some((&input[(first_ptr - input_ptr)..(last_ptr - input_ptr)], rest))
		}
	};
}

macro_rules! group {
	($name:ident: [$($members:ident),+ $(,)?]) => {
		fn $name(input: &str) -> ParseResult<'_> {
			or![$($members,)+](input)
		}
	}
}

type ParseResult<'a> = Option<(&'a str, &'a str)>;

pub trait ParseResultExt {
	fn and_peek(self, peeker: impl_parse!()) -> Self;
	fn and_not(self, peeker: impl_parse!()) -> Self;
	fn succeeded_and_consumed_all(self) -> bool;
}

impl ParseResultExt for ParseResult<'_> {
	fn and_peek(self, peeker: impl_parse!()) -> Self {
		self.filter(|(_ret, rest)| peeker(rest).is_some())
	}

	fn and_not(self, peeker: impl_parse!()) -> Self {
		self.filter(|(_ret, rest)| peeker(rest).is_none())
	}

	fn succeeded_and_consumed_all(self) -> bool {
		self.map_or(false, |(_matched, rest)| rest.is_empty())
	}
}

pub fn eof(input: &str) -> Option<(&str, &str)> {
	if input.is_empty() {
		// to maintain the original pointer
		Some(input.split_at(0))
	} else {
		None
	}
}

fn one_of(chs: &str) -> impl (for<'a> Fn(&'a str) -> ParseResult<'a>) + '_ {
	move |input| {
		let mut chars_orig = input.chars();
		let first = chars_orig.find(|&ch| ch != ',')?;
		let remaining_offset = chars_orig.as_str().as_ptr() as usize - input.as_ptr() as usize;
		if chs.contains(first) {
			Some(input.split_at(remaining_offset))
		} else {
			None
		}
	}
}

fn _slice_until_sub<'a>(container: &'a str, sub: &'_ str) -> &'a str {
	let end = (sub.as_ptr() as usize) - (container.as_ptr() as usize);
	&container[0..end]
}

fn repeat(min: usize, parser: impl_parse!()) -> impl_parse!() {
	move |input| {
		let mut rest = input;
		let mut first = None;
		let mut last = input.as_ptr() as usize;
		let mut num_matches = 0;
		while let Some((this_chunk, new_rest)) = parser(rest) {
			num_matches += 1;
			rest = new_rest;
			first.get_or_insert(this_chunk.as_ptr() as usize);
			last = this_chunk.as_ptr() as usize + this_chunk.len();
		}

		if num_matches < min {
			return None;
		}

		let input_start = input.as_ptr() as usize;
		let first = first.unwrap_or(input_start);
		let first_offset = first - input_start;
		let last_offset = last - input_start;

		Some((&input[first_offset..last_offset], rest))
	}
}

/*
fn eat_commas(input: &str) -> &str {
	let num_commas = input.chars().take_while(|&ch| ch == ',').count();
	&input[num_commas..]
}
*/

fn h(input: &str) -> ParseResult<'_> {
	one_of("'h")(input).and_peek(nucleus)
}

macro_rules! consonant_rule {
	($name:ident, $one_ofs:expr $(, $nots:ident)*) => {
		fn $name(input: &str) -> ParseResult<'_> {
			one_of($one_ofs)(input).and_not(or![h, glide, $name $(, $nots)*])
		}
	};
}

consonant_rule!(b, "bB", unvoiced);
consonant_rule!(c, "cC", s, x, voiced);
consonant_rule!(d, "dD", unvoiced);
consonant_rule!(f, "fF", voiced);
consonant_rule!(g, "gG", unvoiced);
consonant_rule!(j, "jJ", z, unvoiced);
consonant_rule!(k, "kK", x, voiced);
consonant_rule!(l, "lL");
consonant_rule!(m, "mM", z);
consonant_rule!(n, "nN", affricate);
consonant_rule!(p, "pP", voiced);
consonant_rule!(r, "rR");
consonant_rule!(s, "sS", c, voiced);
consonant_rule!(t, "tT", voiced);
consonant_rule!(v, "vV", unvoiced);
consonant_rule!(x, "xX", c, k, voiced);
consonant_rule!(z, "zZ", j, unvoiced);

group!(liquid: [l, r]);
group!(syllabic: [l, m, n, r]);
group!(voiced: [b, d, g, j, v, z]);
group!(unvoiced: [c, f, k, p, s, t, x]);

group!(consonant: [voiced, unvoiced, syllabic]);

fn other(input: &str) -> ParseResult<'_> {
	or![
		p,
		k,
		f,
		x,
		b,
		g,
		v,
		m,
		seq![n, not(liquid)],
		seq![or![t, d], not(l)]
	](input)
}

fn sibilant(input: &str) -> ParseResult<'_> {
	or![c, seq![s, not(x)], seq![or![j, z], not(n), not(liquid)],](input)
}

fn affricate(input: &str) -> ParseResult<'_> {
	or![seq![t, or![c, s]], seq![d, or![j, z]]](input)
}

macro_rules! vowel_rule {
	($name:ident, $one_ofs:expr) => {
		fn $name(input: &str) -> ParseResult<'_> {
			one_of($one_ofs)(input)
		}
	};
}

vowel_rule!(a, "aA");
vowel_rule!(e, "eE");
vowel_rule!(i, "iI");
vowel_rule!(o, "oO");
vowel_rule!(u, "uU");
vowel_rule!(y, "yY");

#[debug_rule]
fn vowel(input: &str) -> ParseResult<'_> {
	one_of("aAeEiIoOuUyY")(input).and_not(nucleus)
}

#[test]
fn test_vowel() {
	assert!(vowel(",,,,,a").succeeded_and_consumed_all());
}

#[debug_rule]
fn diphthong(input: &str) -> ParseResult<'_> {
	or![seq![a, or![i, u]], seq![or![e, o], i]](input).and_not(nucleus)
}

#[test]
fn test_diphthong() {
	assert!(diphthong(",,,,,a,,,,i").succeeded_and_consumed_all());
}

#[debug_rule]
pub fn nucleus(input: &str) -> ParseResult<'_> {
	or![vowel, diphthong, |input| y(input).and_not(nucleus)](input)
}

fn glide(input: &str) -> ParseResult<'_> {
	one_of("iIuU")(input).and_peek(nucleus)
}

fn digit(input: &str) -> ParseResult<'_> {
	one_of("0123456789")(input).and_not(or![h, nucleus])
}

#[debug_rule]
pub fn cmevla(input: &str) -> ParseResult<'_> {
	not!(h(input));
	seq![
		repeat(
			0,
			or![
				nucleus,
				glide,
				h,
				|input| consonant(input).and_not(eof),
				digit
			],
		),
		consonant,
		eof
	](input)
}

#[debug_rule]
pub fn cvcy_lujvo(input: &str) -> ParseResult<'_> {
	or![
		seq![cvc_rafsi, y, opt(h), repeat(0, initial_rafsi), brivla_core],
		seq![stressed_cvc_rafsi, y, short_final_rafsi]
	](input)
}

fn cluster(input: &str) -> ParseResult<'_> {
	repeat(2, consonant)(input)
}

fn initial(input: &str) -> ParseResult<'_> {
	or![affricate, seq![opt(sibilant), opt(other), opt(liquid)]](input)
		.and_not(consonant)
		.and_not(glide)
}

fn onset(input: &str) -> ParseResult<'_> {
	or![h, glide, initial](input)
}

fn stressed(input: &str) -> ParseResult<'_> {
	seq![onset, one_of("AEIOU")](input)
}

#[debug_rule]
pub fn cmavo_form(input: &str) -> ParseResult<'_> {
	or![longest=>
		digit, // e.g., 1 will be treated like pa
		repeat(1, y),
		seq![
			not(h),
			not(cluster),
			onset,
			repeat(0, seq![nucleus, h]),
			or![seq![not(stressed), nucleus], seq![nucleus, not(cluster)]]
		],
	](input)
}

#[test]
fn test_yhy_cmavo() {
	assert_eq!(cmavo_form("y'y"), Some(("y'y", "")));
}

pub fn post_word(input: &str) -> ParseResult<'_> {
	or![eof, seq![not(nucleus), lojban_word]](input)
}

fn initial_pair(input: &str) -> ParseResult<'_> {
	seq![peek(initial), consonant, consonant, not(consonant)](input)
}

fn consonantal_syllable(input: &str) -> ParseResult<'_> {
	seq![consonant, peek(syllabic), coda](input)
}

fn any_syllable(input: &str) -> ParseResult<'_> {
	or![seq![onset, nucleus, opt(coda)], consonantal_syllable](input)
}

fn coda(input: &str) -> ParseResult<'_> {
	or![
		seq![not(any_syllable), consonant, peek(any_syllable)],
		seq![opt(syllabic), opt(consonant), eof]
	](input)
}

fn syllable(input: &str) -> ParseResult<'_> {
	seq![onset, not(y), nucleus, opt(coda)](input)
}

fn stress(input: &str) -> ParseResult<'_> {
	seq![repeat(0, consonant), opt(h), opt(y), syllable, eof](input)
}

fn explicitly_stressed_vowel(input: &str) -> ParseResult<'_> {
	seq![peek(stressed), vowel](input)
}

fn stressed_vowel(input: &str) -> ParseResult<'_> {
	or![explicitly_stressed_vowel, seq![vowel, peek(stress)]](input)
}

fn final_syllable(input: &str) -> ParseResult<'_> {
	seq![
		onset,
		not(y),
		not(stressed),
		nucleus,
		not(cmevla),
		peek(post_word),
	](input)
}

#[debug_rule]
pub fn gismu(input: &str) -> ParseResult<'_> {
	seq![
		or![
			seq![initial_pair, stressed_vowel],
			seq![consonant, stressed_vowel, consonant],
		],
		peek(final_syllable),
		consonant,
		vowel,
		peek(post_word),
	](input)
}

fn cvc_rafsi(input: &str) -> ParseResult<'_> {
	seq![consonant, unstressed_vowel, consonant](input)
}

fn ccv_rafsi(input: &str) -> ParseResult<'_> {
	seq![initial_pair, unstressed_vowel](input)
}

fn unstressed_vowel(input: &str) -> ParseResult<'_> {
	seq![not(stressed), vowel, not(stress)](input)
}

fn unstressed_diphthong(input: &str) -> ParseResult<'_> {
	seq![not(stressed), diphthong, not(stress)](input)
}

fn r_hyphen(input: &str) -> ParseResult<'_> {
	or![seq![r, peek(consonant)], seq![n, peek(r)]](input)
}

fn cvv_rafsi(input: &str) -> ParseResult<'_> {
	seq![
		consonant,
		or![
			seq![unstressed_vowel, h, unstressed_vowel],
			unstressed_diphthong,
		],
		r_hyphen,
	](input)
}

fn y_less_rafsi(input: &str) -> ParseResult<'_> {
	not!(y_rafsi(input));
	not!(stressed_y_rafsi(input));
	not!(hy_rafsi(input));
	not!(stressed_hy_rafsi(input));
	or![cvc_rafsi, ccv_rafsi, cvv_rafsi](input).and_not(h)
}

fn cvv_final_rafsi(input: &str) -> ParseResult<'_> {
	seq![
		consonant,
		stressed_vowel,
		h,
		peek(final_syllable),
		vowel,
		peek(post_word),
	](input)
}

fn stressed_cvc_rafsi(input: &str) -> ParseResult<'_> {
	seq![consonant, stressed_vowel, consonant](input)
}

fn stressed_ccv_rafsi(input: &str) -> ParseResult<'_> {
	seq![initial_pair, stressed_vowel](input)
}

fn stressed_diphthong(input: &str) -> ParseResult<'_> {
	or![
		seq![peek(stressed), diphthong],
		seq![diphthong, peek(stress)],
	](input)
}

fn stressed_cvv_rafsi(input: &str) -> ParseResult<'_> {
	seq![
		consonant,
		or![
			seq![unstressed_vowel, h, stressed_vowel],
			stressed_diphthong,
		],
		opt(r_hyphen)
	](input)
}

fn stressed_y_less_rafsi(input: &str) -> ParseResult<'_> {
	or![
		seq![stressed_cvc_rafsi, not(y)],
		stressed_ccv_rafsi,
		stressed_cvv_rafsi,
	](input)
}

fn long_rafsi(input: &str) -> ParseResult<'_> {
	or![
		seq![initial_pair, unstressed_vowel, consonant],
		seq![consonant, unstressed_vowel, consonant, consonant],
	](input)
}

fn hy_rafsi(input: &str) -> ParseResult<'_> {
	seq![
		or![seq![long_rafsi, vowel], ccv_rafsi, cvv_rafsi],
		h,
		y,
		opt(h)
	](input)
}

fn stressed_hy_rafsi(input: &str) -> ParseResult<'_> {
	seq![
		or![
			seq![long_rafsi, stressed_vowel],
			stressed_ccv_rafsi,
			stressed_cvv_rafsi
		],
		h,
		y,
	](input)
}

fn y_rafsi(input: &str) -> ParseResult<'_> {
	seq![or![long_rafsi, cvc_rafsi], y, opt(h)](input)
}

fn stressed_long_rafsi(input: &str) -> ParseResult<'_> {
	or![
		seq![initial_pair, stressed_vowel, consonant],
		seq![consonant, stressed_vowel, consonant, consonant],
	](input)
}

fn stressed_y_rafsi(input: &str) -> ParseResult<'_> {
	seq![or![stressed_long_rafsi, stressed_cvc_rafsi], y](input)
}

fn short_final_rafsi(input: &str) -> ParseResult<'_> {
	seq![
		peek(final_syllable),
		or![seq![consonant, diphthong], seq![initial_pair, vowel]],
		peek(post_word),
	](input)
}

fn rafsi_string(input: &str) -> ParseResult<'_> {
	seq![
		repeat(0, y_less_rafsi),
		or![
			gismu,
			cvv_final_rafsi,
			seq![stressed_y_less_rafsi, short_final_rafsi],
			y_rafsi,
			stressed_y_rafsi,
			seq![opt(stressed_y_less_rafsi), initial_pair, y],
			hy_rafsi,
			stressed_hy_rafsi,
		]
	](input)
}

fn slinkuhi(input: &str) -> ParseResult<'_> {
	seq![not(rafsi_string), consonant, rafsi_string](input)
}

fn unstressed_syllable(input: &str) -> ParseResult<'_> {
	or![
		seq![not(stressed), syllable, not(stress)],
		consonantal_syllable
	](input)
}

fn brivla_head(input: &str) -> ParseResult<'_> {
	seq![
		not(cmavo),
		not(slinkuhi),
		not(h),
		peek(onset),
		repeat(0, unstressed_syllable),
	](input)
}

fn fuhivla_head(input: &str) -> ParseResult<'_> {
	not!(rafsi_string(input));
	brivla_head(input)
}

fn explicitly_stressed_syllable(input: &str) -> ParseResult<'_> {
	seq![peek(stressed), syllable](input)
}

fn stressed_syllable(input: &str) -> ParseResult<'_> {
	or![explicitly_stressed_syllable, seq![syllable, peek(stress)]](input)
}

#[debug_rule]
pub fn fuhivla(input: &str) -> ParseResult<'_> {
	seq![
		fuhivla_head,
		stressed_syllable,
		repeat(0, consonantal_syllable),
		final_syllable
	](input)
}

fn brivla_rafsi(input: &str) -> ParseResult<'_> {
	seq![
		peek(seq![syllable, repeat(0, consonantal_syllable), syllable]),
		brivla_head,
		h,
		y,
		opt(h),
	](input)
}

fn fuhivla_rafsi(input: &str) -> ParseResult<'_> {
	seq![
		peek(unstressed_syllable),
		fuhivla_head,
		not(h),
		onset,
		y,
		opt(h)
	](input)
}

fn extended_rafsi(input: &str) -> ParseResult<'_> {
	or![brivla_rafsi, fuhivla_rafsi](input)
}

fn stressed_brivla_rafsi(input: &str) -> ParseResult<'_> {
	seq![
		peek(unstressed_syllable),
		brivla_head,
		stressed_syllable,
		h,
		y,
	](input)
}

fn stressed_fuhivla_rafsi(input: &str) -> ParseResult<'_> {
	seq![fuhivla_head, stressed_syllable, not(h), onset, y](input)
}

fn stressed_extended_rafsi(input: &str) -> ParseResult<'_> {
	or![stressed_brivla_rafsi, stressed_fuhivla_rafsi](input)
}

fn any_extended_rafsi(input: &str) -> ParseResult<'_> {
	or![fuhivla, extended_rafsi, stressed_extended_rafsi](input)
}

fn initial_rafsi(input: &str) -> ParseResult<'_> {
	or![
		extended_rafsi,
		y_rafsi,
		seq![
			not(any_extended_rafsi),
			y_less_rafsi,
			not(any_extended_rafsi),
		]
	](input)
}

fn stressed_initial_rafsi(input: &str) -> ParseResult<'_> {
	or![
		stressed_extended_rafsi,
		stressed_y_rafsi,
		stressed_y_less_rafsi
	](input)
}

fn brivla_core(input: &str) -> ParseResult<'_> {
	or![
		fuhivla,
		gismu,
		cvv_final_rafsi,
		seq![stressed_initial_rafsi, short_final_rafsi]
	](input)
}

#[debug_rule]
pub fn explicitly_stressed_gismu_minimal(input: &str) -> ParseResult<'_> {
	seq![
		or![
			seq![initial_pair, explicitly_stressed_vowel],
			seq![consonant, explicitly_stressed_vowel, consonant],
		],
		peek(final_syllable),
		consonant,
		vowel,
	](input)
}

fn final_syllable_minimal(input: &str) -> ParseResult<'_> {
	seq![onset, not(y), not(stressed), nucleus](input)
}

#[debug_rule]
pub fn explicitly_stressed_fuhivla_minimal(input: &str) -> ParseResult<'_> {
	seq![
		fuhivla_head,
		explicitly_stressed_syllable,
		repeat(0, consonantal_syllable),
		final_syllable_minimal
	](input)
}

fn explicitly_stressed_cvv_final_rafsi(input: &str) -> ParseResult<'_> {
	seq![
		consonant,
		explicitly_stressed_vowel,
		h,
		peek(final_syllable),
		vowel,
		peek(post_word),
	](input)
}

fn explicitly_stressed_brivla_rafsi(input: &str) -> ParseResult<'_> {
	seq![
		peek(unstressed_syllable),
		brivla_head,
		explicitly_stressed_syllable,
		h,
		y,
	](input)
}

fn explicitly_stressed_fuhivla_rafsi(input: &str) -> ParseResult<'_> {
	seq![fuhivla_head, explicitly_stressed_syllable, not(h), onset, y](input)
}

fn explicitly_stressed_extended_rafsi(input: &str) -> ParseResult<'_> {
	or![
		explicitly_stressed_brivla_rafsi,
		explicitly_stressed_fuhivla_rafsi
	](input)
}

fn explicitly_stressed_long_rafsi(input: &str) -> ParseResult<'_> {
	or![
		seq![initial_pair, explicitly_stressed_vowel, consonant],
		seq![consonant, explicitly_stressed_vowel, consonant, consonant],
	](input)
}

fn explicitly_stressed_cvc_rafsi(input: &str) -> ParseResult<'_> {
	seq![consonant, explicitly_stressed_vowel, consonant](input)
}

fn explicitly_stressed_y_rafsi(input: &str) -> ParseResult<'_> {
	seq![
		or![
			explicitly_stressed_long_rafsi,
			explicitly_stressed_cvc_rafsi,
		],
		y
	](input)
}

fn explicitly_stressed_ccv_rafsi(input: &str) -> ParseResult<'_> {
	seq![initial_pair, explicitly_stressed_vowel](input)
}

fn explicitly_stressed_diphthong(input: &str) -> ParseResult<'_> {
	seq![peek(stressed), diphthong](input)
}

fn explicitly_stressed_cvv_rafsi(input: &str) -> ParseResult<'_> {
	seq![
		consonant,
		or![
			seq![unstressed_vowel, h, explicitly_stressed_vowel],
			explicitly_stressed_diphthong,
		],
		opt(r_hyphen)
	](input)
}

fn explicitly_stressed_y_less_rafsi(input: &str) -> ParseResult<'_> {
	or![
		seq![explicitly_stressed_cvc_rafsi, not(y)],
		explicitly_stressed_ccv_rafsi,
		explicitly_stressed_cvv_rafsi,
	](input)
}

fn explicitly_stressed_initial_rafsi(input: &str) -> ParseResult<'_> {
	or![
		explicitly_stressed_extended_rafsi,
		explicitly_stressed_y_rafsi,
		explicitly_stressed_y_less_rafsi
	](input)
}

fn explicitly_stressed_brivla_core(input: &str) -> ParseResult<'_> {
	or![
		explicitly_stressed_fuhivla_minimal,
		explicitly_stressed_gismu_minimal,
		explicitly_stressed_cvv_final_rafsi,
		seq![explicitly_stressed_initial_rafsi, short_final_rafsi]
	](input)
}

pub fn explicitly_stressed_brivla_minimal(input: &str) -> ParseResult<'_> {
	seq![repeat(0, initial_rafsi), explicitly_stressed_brivla_core](input)
}

#[debug_rule]
pub fn lujvo(input: &str) -> ParseResult<'_> {
	seq![
		not(gismu),
		not(fuhivla),
		not(cmavo),
		repeat(0, initial_rafsi),
		brivla_core
	](input)
}

/// preconditions: decomposed up to this point (thus implying `!cmavo`), and `!gismu !fuhivla`
#[debug_rule]
pub fn lujvo_minimal(input: &str) -> ParseResult<'_> {
	seq![repeat(0, initial_rafsi), brivla_core](input)
}

#[debug_rule]
pub fn brivla(input: &str) -> ParseResult<'_> {
	or![gismu, fuhivla, seq![not(cmavo), lujvo_minimal]](input)
}

/// preconditions: !cmavo
#[debug_rule]
pub fn brivla_minimal(input: &str) -> ParseResult<'_> {
	or![gismu, fuhivla, lujvo_minimal](input)
}

#[debug_rule]
pub fn lojban_word(input: &str) -> ParseResult<'_> {
	or![
		cmevla,
		cmavo_semi_minimal, // already checked for cmevla
		brivla_minimal,
	](input)
}

#[debug_rule]
pub fn cmavo(input: &str) -> ParseResult<'_> {
	not!(cmevla(input));
	cmavo_semi_minimal(input)
}

/// preconditions: `!cmevla`
/// postconditions: `&post_word`
pub fn cmavo_minimal(input: &str) -> ParseResult<'_> {
	not!(cvcy_lujvo(input));
	cmavo_form(input)
}

/// preconditions: `!cmevla`
pub fn cmavo_semi_minimal(input: &str) -> ParseResult<'_> {
	cmavo_minimal(input).and_peek(post_word)
}
