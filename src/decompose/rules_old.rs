use std::ops::RangeTo;

use nom::branch::{alt, Alt};
use nom::character::complete::{char as one, one_of};
use nom::combinator::{eof, not, opt, peek, recognize};
use nom::error::ParseError;
use nom::multi::{many0_count, many1_count};
use nom::sequence::{tuple, Tuple};
use nom::{IResult, Offset, Slice};

/*
fn template(input: &str) -> IResult<&str, &str> {
	todo!()
}

*/

fn tuple_str<
	I: Clone + Offset + Slice<RangeTo<usize>>,
	O,
	E: ParseError<I>,
	List: Tuple<I, O, E>,
>(
	parsers: List,
) -> impl FnMut(I) -> IResult<I, I, E> {
	recognize(tuple(parsers))
}

fn alt_str<I: Clone + Offset + Slice<RangeTo<usize>>, O, E: ParseError<I>, List: Alt<I, O, E>>(
	parsers: List,
) -> impl FnMut(I) -> IResult<I, I, E> {
	recognize(alt(parsers))
}

fn digit(input: &str) -> IResult<&str, &str> {
	tuple_str((
		many0_count(comma),
		one_of("0123456789"),
		not(h),
		not(nucleus),
	))(input)
}

fn comma(input: &str) -> IResult<&str, char> {
	one(',')(input)
}

fn h(input: &str) -> IResult<&str, &str> {
	tuple_str((many0_count(comma), one_of("'h"), peek(nucleus)))(input)
}

macro_rules! stressed_rule {
	($stressed:ident, $name:ident) => {
		fn $stressed(input: &str) -> IResult<&str, &str> {
			alt((
				tuple_str((peek(stressed), $name)),
				tuple_str(($name, peek(stress))),
			))(input)
		}
	};
}

macro_rules! vowel_rule {
	($it:ident, $it_str:expr) => {
		fn $it(input: &str) -> IResult<&str, &str> {
			tuple_str((many0_count(comma), one_of($it_str)))(input)
		}
	};
}

vowel_rule!(a, "aA");
vowel_rule!(e, "eE");
vowel_rule!(i, "iI");
vowel_rule!(o, "oO");
vowel_rule!(u, "uU");
vowel_rule!(y, "yY");

fn glide(input: &str) -> IResult<&str, &str> {
	tuple_str((alt((i, u)), peek(nucleus)))(input)
}

macro_rules! consonant_rule {
	($it:ident, $it_str:expr $(, $nots:ident)*) => {
		fn $it(input: &str) -> IResult<&str, &str> {
			tuple_str((many0_count(comma), one_of($it_str), not(h), not(glide), not($it) $(, not($nots))*))(input)
		}
	}
}

fn unvoiced(input: &str) -> IResult<&str, &str> {
	alt((c, f, k, p, s, t, x))(input)
}

fn voiced(input: &str) -> IResult<&str, &str> {
	alt((b, d, g, j, v, z))(input)
}

fn syllabic(input: &str) -> IResult<&str, &str> {
	alt((l, m, n, r))(input)
}

consonant_rule!(b, "bB", unvoiced);
consonant_rule!(c, "cC", s, x, voiced);
consonant_rule!(d, "dD", voiced);
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

fn affricate(input: &str) -> IResult<&str, &str> {
	alt_str((tuple((t, c)), tuple((t, s)), tuple((d, j)), tuple((d, z))))(input)
}

fn sibilant(input: &str) -> IResult<&str, &str> {
	alt((
		c,
		tuple_str((s, not(x))),
		tuple_str((alt((j, z)), not(n), not(liquid))),
	))(input)
}

fn other(input: &str) -> IResult<&str, &str> {
	alt((p, tuple_str((t, not(l)))))(input)
}

fn liquid(input: &str) -> IResult<&str, &str> {
	alt((l, r))(input)
}

fn consonant(input: &str) -> IResult<&str, &str> {
	alt((voiced, unvoiced, syllabic))(input)
}

fn initial(input: &str) -> IResult<&str, &str> {
	tuple_str((
		alt((
			affricate,
			tuple_str((opt(sibilant), opt(other), opt(liquid))),
		)),
		not(consonant),
		not(glide),
	))(input)
}

fn onset(input: &str) -> IResult<&str, &str> {
	alt((h, glide, initial))(input)
}

fn vowel(input: &str) -> IResult<&str, &str> {
	tuple_str((alt((a, e, i, o, u)), not(nucleus)))(input)
}

fn diphthong(input: &str) -> IResult<&str, &str> {
	tuple_str((
		alt((
			tuple_str((a, i, not(i))),
			tuple_str((a, u, not(u))),
			tuple_str((e, i, not(i))),
			tuple_str((o, i, not(i))),
		)),
		not(nucleus),
	))(input)
}

fn nucleus(input: &str) -> IResult<&str, &str> {
	alt((vowel, diphthong, tuple_str((y, not(nucleus)))))(input)
}

fn coda(input: &str) -> IResult<&str, &str> {
	alt((
		tuple_str((not(any_syllable), consonant, peek(any_syllable))),
		tuple_str((opt(syllabic), opt(consonant), peek(pause))),
	))(input)
}

fn consonantal_syllable(input: &str) -> IResult<&str, &str> {
	tuple_str((consonant, peek(syllabic), coda))(input)
}

fn any_syllable(input: &str) -> IResult<&str, &str> {
	alt((tuple_str((onset, nucleus, opt(coda))), consonantal_syllable))(input)
}

fn space_char(input: &str) -> IResult<&str, char> {
	one_of(".\t\n\r?! ")(input)
}

fn pause(input: &str) -> IResult<&str, &str> {
	alt((
		tuple_str((many0_count(comma), many1_count(space_char))),
		eof,
	))(input)
}

fn jbocme(input: &str) -> IResult<&str, &str> {
	tuple((
		peek(zifcme),
		recognize(many0_count(alt((any_syllable, digit)))),
		peek(pause),
	))(input)
	.map(|(rest, (_, jbocme, _))| (rest, jbocme))
}

fn zifcme(input: &str) -> IResult<&str, &str> {
	tuple_str((
		not(h),
		many0_count(alt((
			nucleus,
			glide,
			h,
			tuple_str((consonant, not(pause))),
			digit,
		))),
		consonant,
		peek(pause),
	))(input)
}

fn cmevla(input: &str) -> IResult<&str, &str> {
	alt((jbocme, zifcme))(input)
}

fn stressed(input: &str) -> IResult<&str, &str> {
	tuple_str((onset, many0_count(comma), one_of("AEIOU")))(input)
}

fn syllable(input: &str) -> IResult<&str, &str> {
	tuple_str((onset, not(y), nucleus, opt(coda)))(input)
}

fn stress(input: &str) -> IResult<&str, &str> {
	tuple_str((many0_count(consonant), opt(h), opt(y), syllable, pause))(input)
}

fn unstressed_vowel(input: &str) -> IResult<&str, &str> {
	tuple_str((not(stressed), vowel, not(stress)))(input)
}

fn cvc_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((consonant, unstressed_vowel, consonant))(input)
}

fn unstressed_syllable(input: &str) -> IResult<&str, &str> {
	alt((
		tuple_str((not(stressed), syllable, stressed)),
		consonantal_syllable,
	))(input)
}

stressed_rule!(stressed_vowel, vowel);

fn gismu(input: &str) -> IResult<&str, &str> {
	tuple_str((
		alt((
			tuple_str((initial_pair, stressed_vowel)),
			tuple_str((consonant, stressed_vowel, consonant)),
		)),
		peek(final_syllable),
		consonant,
		vowel,
		peek(post_word),
	))(input)
}

fn cvv_final_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((
		consonant,
		stressed_vowel,
		h,
		peek(final_syllable),
		vowel,
		peek(post_word),
	))(input)
}

fn stressed_ccv_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((initial_pair, stressed_vowel))(input)
}

stressed_rule!(stressed_diphthong, diphthong);

fn r_hyphen(input: &str) -> IResult<&str, &str> {
	alt((tuple_str((r, peek(consonant))), tuple_str((n, peek(r)))))(input)
}

fn stressed_cvv_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((
		consonant,
		alt((
			tuple_str((unstressed_vowel, h, stressed_vowel)),
			stressed_diphthong,
		)),
		opt(r_hyphen),
	))(input)
}

fn stressed_y_less_rafsi(input: &str) -> IResult<&str, &str> {
	alt((
		tuple_str((stressed_cvc_rafsi, not(y))),
		stressed_ccv_rafsi,
		stressed_cvv_rafsi,
	))(input)
}

fn stressed_long_rafsi(input: &str) -> IResult<&str, &str> {
	alt((
		tuple_str((initial_pair, stressed_vowel, consonant)),
		tuple_str((consonant, stressed_vowel, consonant, consonant)),
	))(input)
}

fn stressed_y_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((alt((stressed_long_rafsi, stressed_cvc_rafsi)), y))(input)
}

fn initial_pair(input: &str) -> IResult<&str, &str> {
	tuple_str((peek(initial), consonant, consonant, not(consonant)))(input)
}

fn long_rafsi(input: &str) -> IResult<&str, &str> {
	alt((
		tuple_str((initial_pair, unstressed_vowel, consonant)),
		tuple_str((consonant, unstressed_vowel, consonant, consonant)),
	))(input)
}

fn ccv_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((initial_pair, unstressed_vowel))(input)
}

fn unstressed_diphthong(input: &str) -> IResult<&str, &str> {
	tuple_str((not(stressed), diphthong, not(stress)))(input)
}

fn cvv_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((
		consonant,
		alt((
			tuple_str((unstressed_vowel, h, unstressed_vowel)),
			unstressed_diphthong,
		)),
		opt(r_hyphen),
	))(input)
}

fn hy_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((
		alt((tuple_str((long_rafsi, vowel)), ccv_rafsi, cvv_rafsi)),
		h,
		y,
		opt(h),
	))(input)
}

fn stressed_hy_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((
		alt((
			tuple_str((long_rafsi, stressed_vowel)),
			stressed_ccv_rafsi,
			stressed_cvv_rafsi,
		)),
		h,
		y,
	))(input)
}

fn rafsi_string(input: &str) -> IResult<&str, &str> {
	tuple_str((
		many0_count(y_less_rafsi),
		alt((
			gismu,
			cvv_final_rafsi,
			tuple_str((stressed_y_less_rafsi, short_final_rafsi)),
			y_rafsi,
			stressed_y_rafsi,
			tuple_str((opt(stressed_y_less_rafsi), initial_pair, y)),
			hy_rafsi,
			stressed_hy_rafsi,
		)),
	))(input)
}

fn slinkuhi(input: &str) -> IResult<&str, &str> {
	tuple_str((not(rafsi_string), consonant, rafsi_string))(input)
}

fn brivla_head(input: &str) -> IResult<&str, &str> {
	tuple_str((
		not(cmavo),
		not(slinkuhi),
		not(h),
		peek(onset),
		many0_count(unstressed_syllable),
	))(input)
}

fn brivla_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((
		peek(tuple((
			syllable,
			many0_count(consonantal_syllable),
			syllable,
		))),
		brivla_head,
		h,
		y,
		opt(h),
	))(input)
}

fn fuhivla_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((
		peek(unstressed_syllable),
		fuhivla_head,
		not(h),
		onset,
		y,
		opt(h),
	))(input)
}

fn extended_rafsi(input: &str) -> IResult<&str, &str> {
	alt((brivla_rafsi, fuhivla_rafsi))(input)
}

fn fuhivla_head(input: &str) -> IResult<&str, &str> {
	tuple_str((not(rafsi_string), brivla_head))(input)
}

fn stressed_syllable(input: &str) -> IResult<&str, &str> {
	alt((
		tuple_str((peek(stressed), syllable)),
		tuple_str((syllable, peek(stress))),
	))(input)
}

fn final_syllable(input: &str) -> IResult<&str, &str> {
	tuple_str((
		onset,
		not(y),
		not(stressed),
		nucleus,
		not(cmevla),
		peek(post_word),
	))(input)
}

pub fn fuhivla(input: &str) -> IResult<&str, &str> {
	tuple_str((
		fuhivla_head,
		stressed_syllable,
		many0_count(consonantal_syllable),
		final_syllable,
	))(input)
}

fn stressed_brivla_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((
		peek(unstressed_syllable),
		brivla_head,
		stressed_syllable,
		h,
		y,
	))(input)
}

fn stressed_fuhivla_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((fuhivla_head, stressed_syllable, not(h), onset, y))(input)
}

fn stressed_extended_rafsi(input: &str) -> IResult<&str, &str> {
	alt((stressed_brivla_rafsi, stressed_fuhivla_rafsi))(input)
}

fn any_extended_rafsi(input: &str) -> IResult<&str, &str> {
	alt((fuhivla, extended_rafsi, stressed_extended_rafsi))(input)
}

fn y_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((alt((long_rafsi, cvc_rafsi)), y, opt(h)))(input)
}

fn y_less_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((
		not(y_rafsi),
		not(stressed_y_rafsi),
		not(hy_rafsi),
		not(stressed_hy_rafsi),
		alt((cvc_rafsi, ccv_rafsi, cvv_rafsi)),
		not(h),
	))(input)
}

fn initial_rafsi(input: &str) -> IResult<&str, &str> {
	alt((
		extended_rafsi,
		y_rafsi,
		tuple_str((
			not(any_extended_rafsi),
			y_less_rafsi,
			not(any_extended_rafsi),
		)),
	))(input)
}

fn stressed_initial_rafsi(input: &str) -> IResult<&str, &str> {
	alt((
		stressed_extended_rafsi,
		stressed_y_rafsi,
		stressed_y_less_rafsi,
	))(input)
}

fn brivla_core(input: &str) -> IResult<&str, &str> {
	alt((
		fuhivla,
		gismu,
		cvv_final_rafsi,
		tuple_str((stressed_initial_rafsi, short_final_rafsi)),
	))(input)
}

fn stressed_cvc_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((consonant, stressed_vowel, consonant))(input)
}

fn short_final_rafsi(input: &str) -> IResult<&str, &str> {
	tuple_str((
		peek(final_syllable),
		alt((
			tuple_str((consonant, diphthong)),
			alt((initial_pair, vowel)),
		)),
		peek(post_word),
	))(input)
}

fn cvcy_lujvo(input: &str) -> IResult<&str, &str> {
	alt((
		tuple_str((
			cvc_rafsi,
			y,
			opt(h),
			many0_count(initial_rafsi),
			brivla_core,
		)),
		tuple_str((stressed_cvc_rafsi, y, short_final_rafsi)),
	))(input)
}

fn cluster(input: &str) -> IResult<&str, &str> {
	tuple_str((consonant, many1_count(consonant)))(input)
}

fn cmavo_form(input: &str) -> IResult<&str, &str> {
	alt((
		tuple_str((
			not(h),
			not(cluster),
			onset,
			many0_count(tuple_str((nucleus, h))),
			alt((
				tuple_str((not(stressed), nucleus)),
				tuple_str((nucleus, not(cluster))),
			)),
		)),
		recognize(many1_count(y)),
		digit,
	))(input)
}

pub fn lujvo(input: &str) -> IResult<&str, &str> {
	tuple_str((
		not(gismu),
		not(fuhivla),
		not(cmavo),
		many0_count(initial_rafsi),
		brivla_core,
	))(input)
}

fn brivla(input: &str) -> IResult<&str, &str> {
	alt((gismu, fuhivla, lujvo))(input)
}

fn lojban_word(input: &str) -> IResult<&str, &str> {
	alt((cmevla, cmavo, brivla))(input)
}

fn post_word(input: &str) -> IResult<&str, &str> {
	alt((pause, tuple_str((not(nucleus), lojban_word))))(input)
}

pub fn cmavo(input: &str) -> IResult<&str, &str> {
	tuple((not(cmevla), not(cvcy_lujvo), cmavo_form, peek(post_word)))(input)
		.map(|(rest, (_, _, cmavo, _))| (rest, cmavo))
}
