use nom::branch::alt;
use nom::combinator::{all_consuming, cut, map, not, opt};
use nom::sequence::tuple;
use nom::Parser;

use crate::lex::Token;

pub mod cst;
pub mod error;

pub use error::Error;

pub type ParseResult<'a, T> = nom::IResult<&'a [Token], T, error::WithLocation<'a>>;

fn many0<'a, T>(
	parser: impl Parser<&'a [Token], T, error::WithLocation<'a>>,
) -> impl Parser<&'a [Token], Box<[T]>, error::WithLocation<'a>> {
	map(nom::multi::many0(parser), Vec::into_boxed_slice)
}

fn many1<'a, T>(
	parser: impl Parser<&'a [Token], T, error::WithLocation<'a>>,
) -> impl Parser<&'a [Token], Box<[T]>, error::WithLocation<'a>> {
	map(nom::multi::many1(parser), Vec::into_boxed_slice)
}

fn box_<'a, T>(
	parser: impl Parser<&'a [Token], T, error::WithLocation<'a>>,
) -> impl FnMut(&'a [Token]) -> ParseResult<'a, Box<T>> {
	map(parser, Box::new)
}

fn selmaho_raw<T: cst::SelmahoTypeRaw>(input: &[Token]) -> ParseResult<'_, T> {
	let mut input = input.iter();
	T::try_from(input.next().copied())
		.map(|matched| (input.as_slice(), matched))
		.map_err(|error| {
			nom::Err::Error(error::WithLocation {
				location: input.as_slice(),
				error,
			})
		})
}

fn selmaho<T: cst::SelmahoType>(input: &[Token]) -> ParseResult<'_, T> {
	let (input, bahe) = many0(selmaho_raw::<cst::Bahe>).parse(input)?;
	let (rest, mut matched) = selmaho_raw::<T>(input)?;
	matched.set_bahe(bahe);
	Ok((rest, matched))
}

macro_rules! selmaho {
	($name:ident) => {
		selmaho::<cst::$name>
	};
}

/// Whether to set `should_cut` is a bit of a tricky question.
/// It is complicated by the elision of elidable terminators.
/// For example, `lo broda joi lo brode` is perfectly acceptable and implies a `KU` before `joi`.
/// However, if `should_cut` was set to true, the failure to parse a selbri after the `joi` (since selbri can also be connected with `JOI`) would have caused a parse *failure* (not an error).
/// Thus, in that situation `should_cut` must be false.
fn separated<'a, Item, Separator>(
	item: impl Parser<&'a [Token], Item, error::WithLocation<'a>> + Clone,
	separator: impl Parser<&'a [Token], Separator, error::WithLocation<'a>> + Clone,
	should_cut: bool,
) -> impl Parser<&'a [Token], cst::Separated<Item, Separator>, error::WithLocation<'a>> + Clone {
	move |input| {
		map(
			tuple((
				item.clone(),
				many0(tuple((separator.clone(), |input| {
					if should_cut {
						cut(item.clone())(input)
					} else {
						item.clone().parse(input)
					}
				}))),
			)),
			|(first, rest)| cst::Separated { first, rest },
		)(input)
	}
}

pub fn parse(input: &[Token]) -> Result<cst::Root, error::WithLocation<'_>> {
	nom::Finish::finish(text(input)).map(|(rest, root)| {
		debug_assert!(rest.is_empty());
		root
	})
}

fn text(input: &[Token]) -> ParseResult<'_, cst::Text> {
	map(
		all_consuming(tuple((
			opt(selmaho!(I)),
			separated(sentence, selmaho!(I), true),
			opt(selmaho!(Faho)),
		))),
		|(initial_i, sentences, faho)| cst::Text {
			initial_i,
			sentences,
			faho,
		},
	)(input)
}

fn sentence(mut input: &[Token]) -> ParseResult<'_, cst::Sentence> {
	let mut args = Vec::new();

	macro_rules! args {
		() => {
			while let Ok((new_input, arg)) = arg(input) {
				input = new_input;
				args.push(arg);
			}
		};
	}

	let (new_input, prenexes) = many0(prenex).parse(input)?;
	input = new_input;

	args!();

	let (new_input, cu) = opt(selmaho!(Cu))(input)?;
	input = new_input;

	// require selbri if cu is found
	let (new_input, selbri) = if cu.is_some() {
		map(cut(selbri), Some)(input)?
	} else {
		opt(selbri)(input)?
	};
	let selbri = selbri.map(|selbri| (cu, selbri));
	input = new_input;

	let num_args_before_selbri = args.len();

	// we only need to read more sumti if we encountered a selbri
	if selbri.is_some() {
		args!();
	}

	Ok((
		input,
		cst::Sentence {
			prenexes,
			selbri,
			args: args.into_boxed_slice(),
			num_args_before_selbri,
		},
	))
}

fn prenex(input: &[Token]) -> ParseResult<'_, cst::Prenex> {
	map(tuple((many0(arg), selmaho!(Zohu))), |(terms, zohu)| {
		cst::Prenex { terms, zohu }
	})(input)
}

fn selbri(input: &[Token]) -> ParseResult<'_, cst::Selbri> {
	map(tuple((many0(selmaho!(Na)), selbri1)), |(na, components)| {
		cst::Selbri { na, components }
	})(input)
}

fn selbri1(input: &[Token]) -> ParseResult<'_, cst::Selbri1> {
	separated(selbri2, selmaho!(Co), true).parse(input)
}

fn selbri2(input: &[Token]) -> ParseResult<'_, cst::Selbri2> {
	many1(selbri3).parse(input)
}

fn selbri3(input: &[Token]) -> ParseResult<'_, cst::Selbri3> {
	separated(selbri4, joik_jek, false).parse(input)
}

fn selbri4(input: &[Token]) -> ParseResult<'_, cst::Selbri4> {
	separated(
		selbri5,
		|input| tuple((joik_jek, selmaho!(Bo)))(input),
		false,
	)
	.parse(input)
}

fn selbri5(input: &[Token]) -> ParseResult<'_, cst::Selbri5> {
	separated(selbri6, selmaho!(Bo), false).parse(input)
}

fn selbri6(input: &[Token]) -> ParseResult<'_, cst::Selbri6> {
	alt((
		map(
			tuple((
				opt(selmaho!(Nahe)),
				selmaho!(Guha),
				cut(selbri),
				selmaho!(Gi),
				selbri6,
			)),
			|(nahe, guha, first, gi, second)| cst::Selbri6::Connected {
				nahe,
				guha,
				first: Box::new(first),
				gi,
				second: Box::new(second),
			},
		),
		map(tanru_unit, cst::Selbri6::NotConnected),
	))(input)
}

fn tanru_unit(input: &[Token]) -> ParseResult<'_, cst::TanruUnit> {
	separated(tanru_unit_1, selmaho!(Cei), true).parse(input)
}

fn tanru_unit_1(input: &[Token]) -> ParseResult<'_, cst::TanruUnit1> {
	map(
		tuple((many0(before_tanru_unit), tanru_unit_2, opt(bound_arguments))),
		|(before, inner, bound_arguments)| cst::TanruUnit1 {
			before,
			inner,
			bound_arguments,
		},
	)(input)
}

fn before_tanru_unit(input: &[Token]) -> ParseResult<'_, cst::BeforeTanruUnit> {
	alt((
		map(tuple((selmaho!(Jai), opt(tag_word))), |(jai, tag)| {
			cst::BeforeTanruUnit::Jai { jai, tag }
		}),
		map(selmaho!(Nahe), cst::BeforeTanruUnit::Nahe),
		map(selmaho!(Se), cst::BeforeTanruUnit::Se),
	))(input)
}

fn joik_jek(input: &[Token]) -> ParseResult<'_, cst::JoikJek> {
	map(
		tuple((
			opt(selmaho!(Na)),
			opt(selmaho!(Se)),
			joik_jek_word,
			opt(selmaho!(Nai)),
		)),
		|(na, se, word, nai)| cst::JoikJek { na, se, word, nai },
	)(input)
}

fn joik_jek_word(input: &[Token]) -> ParseResult<'_, cst::JoikJekWord> {
	alt((
		map(selmaho!(Ja), cst::JoikJekWord::Ja),
		map(selmaho!(Joi), cst::JoikJekWord::Joi),
	))(input)
}

/*
fn selbri_component(input: &[Token]) -> ParseResult<'_, cst::SelbriComponent> {
	map(
		tuple((
			many0(before_selbri_component),
			selbri_word,
			opt(bound_arguments),
		)),
		|(before, word, bound_arguments)| cst::SelbriComponent {
			before,
			word,
			bound_arguments,
		},
	)(input)
}

fn before_selbri_component(input: &[Token]) -> ParseResult<'_, cst::BeforeTanruUnit> {
	alt((
		map(selmaho!(Jai), cst::BeforeTanruUnit::Jai),
		map(selmaho!(Nahe), cst::BeforeTanruUnit::Nahe),
		map(selmaho!(Se), cst::BeforeTanruUnit::Se),
	))(input)
}
*/

fn tanru_unit_2(input: &[Token]) -> ParseResult<'_, cst::TanruUnit2> {
	alt((
		map(
			tuple((selmaho!(Ke), cut(selbri2), opt(selmaho!(Kehe)))),
			|(ke, group, kehe)| cst::TanruUnit2::GroupedTanru { ke, group, kehe },
		),
		map(selmaho!(Gismu), cst::TanruUnit2::Gismu),
		map(selmaho!(Lujvo), cst::TanruUnit2::Lujvo),
		map(selmaho!(Fuhivla), cst::TanruUnit2::Fuhivla),
		map(
			tuple((selmaho!(Goha), opt(selmaho!(Raho)))),
			|(goha, raho)| cst::TanruUnit2::Goha { goha, raho },
		),
		map(
			tuple((selmaho!(Nuha), cut(mekso_operator))),
			|(nuha, operator)| cst::TanruUnit2::Nuha { nuha, operator },
		),
		map(
			tuple((
				separated(
					|input| tuple((selmaho!(Nu), opt(selmaho!(Nai)))).parse(input),
					joik_jek,
					true,
				),
				cut(sentence),
				opt(selmaho!(Kei)),
			)),
			|(nus, inner, kei)| cst::TanruUnit2::Nu {
				nus,
				inner: Box::new(inner),
				kei,
			},
		),
		map(
			tuple((selmaho!(Me), cut(sumti), opt(selmaho!(Mehu)))),
			|(me, inner, mehu)| cst::TanruUnit2::Me {
				me,
				inner: Box::new(inner),
				mehu,
			},
		),
		map(
			tuple((many1(number_rest), selmaho!(Moi))),
			|(number, moi)| cst::TanruUnit2::Moi(number, moi),
		),
	))(input)
}

fn bound_arguments(input: &[Token]) -> ParseResult<'_, cst::BoundArguments> {
	map(
		tuple((
			selmaho!(Be),
			cut(separated(arg, selmaho!(Bei), true)),
			opt(selmaho!(Beho)),
		)),
		|(be, args, beho)| cst::BoundArguments { be, args, beho },
	)(input)
}

fn arg(input: &[Token]) -> ParseResult<'_, cst::Arg> {
	alt((
		map(tuple((tag_word, selmaho!(Ku))), |(tag, ku)| {
			cst::Arg::TagKu { tag, ku }
		}),
		map(tag, cst::Arg::Tag),
		map(tuple((opt(selmaho!(Fa)), sumti)), |(fa, sumti)| {
			cst::Arg::Sumti { fa, sumti }
		}),
	))(input)
}

fn tag(input: &[Token]) -> ParseResult<'_, cst::Tag> {
	map(
		tuple((separated(tag_word, joik_jek, false), tag_value)),
		|(words, value)| cst::Tag { words, value },
	)(input)
}

fn tag_word(input: &[Token]) -> ParseResult<'_, cst::TagWord> {
	alt((
		map(
			tuple((opt(selmaho!(Se)), selmaho!(Bai), opt(selmaho!(Nai)))),
			|(se, bai, nai)| cst::TagWord::Bai { se, bai, nai },
		),
		map(converted_tag_word, cst::TagWord::Converted),
	))(input)
}

fn converted_tag_word(input: &[Token]) -> ParseResult<'_, cst::Selbri> {
	map(
		tuple((selmaho!(Fiho), selbri, opt(selmaho!(Fehu)))),
		|(_fiho, selbri, _fehu)| selbri,
	)(input)
}

fn tag_value(input: &[Token]) -> ParseResult<'_, Option<cst::Sumti>> {
	// using `map` rather than `value` to avoid the `Clone` bound
	alt((map(selmaho!(Ku), |_| None), map(sumti, Some)))(input)
}

fn sumti(input: &[Token]) -> ParseResult<'_, cst::Sumti> {
	map(
		tuple((
			separated(
				separated(
					sumti_component_outer,
					|input| tuple((sumti_connective, selmaho!(Bo)))(input),
					false,
				),
				sumti_connective,
				false,
			),
			opt(vuho_relative),
		)),
		|(inner, vuho_relative)| cst::Sumti {
			inner,
			vuho_relative,
		},
	)(input)
}

fn sumti_component_outer(input: &[Token]) -> ParseResult<'_, cst::SumtiComponentOuter> {
	alt((
		map(
			tuple((opt(quantifier), sumti_component, opt(relative_clauses))),
			|(quantifier, inner, relative_clauses)| cst::SumtiComponentOuter::Normal {
				quantifier,
				inner,
				relative_clauses,
			},
		),
		map(
			tuple((quantifier, selbri, opt(selmaho!(Ku)), opt(relative_clauses))),
			|(quantifier, inner, ku, relative_clauses)| cst::SumtiComponentOuter::SelbriShorthand {
				quantifier,
				inner: Box::new(inner),
				ku,
				relative_clauses,
			},
		),
	))(input)
}

fn quantifier(input: &[Token]) -> ParseResult<'_, cst::Quantifier> {
	alt((
		map(
			tuple((selmaho!(Vei), cut(mekso), opt(selmaho!(Veho)))),
			|(vei, mekso, veho)| cst::Quantifier::Mekso { vei, mekso, veho },
		),
		map(
			tuple((number, not(selmaho!(Moi)), opt(selmaho!(Boi)))),
			|(number, _, boi)| cst::Quantifier::Number { number, boi },
		),
	))(input)
}

fn mekso(input: &[Token]) -> ParseResult<'_, cst::Mekso> {
	todo!("mekso")
}

fn mekso_operator(input: &[Token]) -> ParseResult<'_, cst::MeksoOperator> {
	todo!("mekso")
}

fn number(input: &[Token]) -> ParseResult<'_, cst::Number> {
	map(
		tuple((selmaho!(Pa), many0(number_rest))),
		|(first, rest)| cst::Number { first, rest },
	)(input)
}

fn number_rest(input: &[Token]) -> ParseResult<'_, cst::NumberRest> {
	alt((
		map(selmaho!(Pa), cst::NumberRest::Pa),
		map(lerfu_word, cst::NumberRest::Lerfu),
	))(input)
}

fn lerfu_string(input: &[Token]) -> ParseResult<'_, cst::LerfuString> {
	map(tuple((lerfu_word, many0(number_rest))), |(first, rest)| {
		cst::LerfuString { first, rest }
	})(input)
}

fn lerfu_word(input: &[Token]) -> ParseResult<'_, cst::LerfuWord> {
	alt((
		map(selmaho!(By), cst::LerfuWord::By),
		map(tuple((selmaho!(Lau), cut(selmaho!(By)))), |(lau, by)| {
			cst::LerfuWord::Lau { lau, by }
		}),
		map(
			tuple((selmaho!(Tei), cut(lerfu_string), cut(selmaho!(Foi)))),
			|(tei, inner, foi)| cst::LerfuWord::Tei {
				tei,
				inner: Box::new(inner),
				foi,
			},
		),
	))(input)
}

fn vuho_relative(input: &[Token]) -> ParseResult<'_, cst::VuhoRelative> {
	map(
		tuple((selmaho!(Vuho), cut(relative_clauses))),
		|(vuho, relative_clauses)| cst::VuhoRelative {
			vuho,
			relative_clauses,
		},
	)(input)
}

fn relative_clauses(input: &[Token]) -> ParseResult<'_, cst::RelativeClauses> {
	separated(relative_clause, selmaho!(Zihe), true).parse(input)
}

fn relative_clause(input: &[Token]) -> ParseResult<'_, cst::RelativeClause> {
	alt((
		map(goi_relative_clause, cst::RelativeClause::Goi),
		map(noi_relative_clause, cst::RelativeClause::Noi),
	))(input)
}

fn goi_relative_clause(input: &[Token]) -> ParseResult<'_, cst::GoiRelativeClause> {
	map(
		tuple((selmaho!(Goi), cut(arg), opt(selmaho!(Gehu)))),
		|(goi, inner, gehu)| cst::GoiRelativeClause {
			goi,
			inner: Box::new(inner),
			gehu,
		},
	)(input)
}

fn noi_relative_clause(input: &[Token]) -> ParseResult<'_, cst::NoiRelativeClause> {
	map(
		tuple((selmaho!(Noi), cut(sentence), opt(selmaho!(Kuho)))),
		|(noi, sentence, kuho)| cst::NoiRelativeClause {
			noi,
			sentence: Box::new(sentence),
			kuho,
		},
	)(input)
}

fn sumti_component(input: &[Token]) -> ParseResult<'_, cst::SumtiComponent> {
	alt((
		map(selmaho!(Koha), cst::SumtiComponent::Koha),
		// it is important that this is checked before `la_sumti` because `la_sumti` `cut`s on `cmevla`
		map(gadri_sumti, cst::SumtiComponent::Gadri),
		map(la_sumti, cst::SumtiComponent::La),
		map(lohu_sumti, cst::SumtiComponent::Lohu),
		map(lu_sumti, cst::SumtiComponent::Lu),
		map(modified_sumti, cst::SumtiComponent::Modified),
		map(
			tuple((lerfu_string, opt(selmaho!(Boi)))),
			|(lerfu_string, boi)| cst::SumtiComponent::LerfuString(lerfu_string, boi),
		),
		map(zo_sumti, cst::SumtiComponent::Zo),
		map(zoi_sumti, cst::SumtiComponent::Zoi),
		map(
			tuple((selmaho!(Li), mekso, opt(selmaho!(Loho)))),
			|(li, mekso, loho)| cst::SumtiComponent::Li(li, mekso, loho),
		),
	))(input)
}

fn lohu_sumti(input: &[Token]) -> ParseResult<'_, cst::LohuSumti> {
	map(
		tuple((selmaho!(Lohu), many0(token), selmaho!(Lehu))),
		|(lohu, inner, lehu)| cst::LohuSumti { lohu, inner, lehu },
	)(input)
}

fn lu_sumti(input: &[Token]) -> ParseResult<'_, cst::LuSumti> {
	map(
		tuple((selmaho!(Lu), box_(text), opt(selmaho!(Lihu)))),
		|(lu, text, lihu)| cst::LuSumti { lu, text, lihu },
	)(input)
}

fn modified_sumti(input: &[Token]) -> ParseResult<'_, cst::ModifiedSumti> {
	map(
		tuple((
			sumti_modifier,
			opt(relative_clauses),
			box_(sumti),
			opt(selmaho!(Luhu)),
		)),
		|(modifier, relative_clauses, sumti, luhu)| cst::ModifiedSumti {
			modifier,
			relative_clauses,
			sumti,
			luhu,
		},
	)(input)
}

fn sumti_modifier(input: &[Token]) -> ParseResult<'_, cst::SumtiModifier> {
	alt((
		map(selmaho!(Lahe), cst::SumtiModifier::Lahe),
		map(tuple((selmaho!(Nahe), selmaho!(Bo))), |(nahe, bo)| {
			cst::SumtiModifier::NaheBo(nahe, bo)
		}),
	))(input)
}

fn gadri_sumti(input: &[Token]) -> ParseResult<'_, cst::GadriSumti> {
	map(
		tuple((
			gadri,
			opt(box_(sumti_component)),
			opt(relative_clauses),
			gadri_sumti_inner,
			opt(selmaho!(Ku)),
		)),
		|(gadri, pe_shorthand, relative_clauses, inner, ku)| cst::GadriSumti {
			gadri,
			pe_shorthand,
			relative_clauses,
			inner,
			ku,
		},
	)(input)
}

fn gadri(input: &[Token]) -> ParseResult<'_, cst::Gadri> {
	alt((
		map(selmaho!(Le), cst::Gadri::Le),
		map(selmaho!(La), cst::Gadri::La),
	))(input)
}

fn gadri_sumti_inner(input: &[Token]) -> ParseResult<'_, cst::GadriSumtiInner> {
	alt((
		map(tuple((quantifier, box_(sumti))), |(quantifier, sumti)| {
			cst::GadriSumtiInner::Sumti(quantifier, sumti)
		}),
		map(
			tuple((opt(quantifier), box_(selbri), opt(relative_clauses))),
			|(quantifier, selbri, relative_clauses)| {
				cst::GadriSumtiInner::Selbri(quantifier, selbri, relative_clauses)
			},
		),
	))(input)
}

fn la_sumti(input: &[Token]) -> ParseResult<'_, cst::LaSumti> {
	map(
		tuple((selmaho!(La), cut(many1(selmaho!(Cmevla))))),
		|(la, inner)| cst::LaSumti { la, inner },
	)(input)
}

fn zo_sumti(input: &[Token]) -> ParseResult<'_, cst::ZoSumti> {
	map(tuple((selmaho!(Zo), cut(token))), |(zo, quoted)| {
		cst::ZoSumti { zo, quoted }
	})(input)
}

fn zoi_sumti(input: &[Token]) -> ParseResult<'_, cst::ZoiSumti> {
	map(
		tuple((selmaho!(Zoi), span, span, span)),
		|(zoi, starting_delimiter, text, ending_delimiter)| cst::ZoiSumti {
			zoi,
			starting_delimiter,
			text,
			ending_delimiter,
		},
	)(input)
}

fn sumti_connective(input: &[Token]) -> ParseResult<'_, cst::SumtiConnective> {
	alt((
		map(selmaho!(A), cst::SumtiConnective::A),
		map(selmaho!(Joi), cst::SumtiConnective::Joi),
	))(input)
}

fn token(input: &[Token]) -> ParseResult<'_, crate::lex::Token> {
	let mut input = input.iter();
	input
		.next()
		.map(|&token| (input.as_slice(), token))
		.ok_or(nom::Err::Error(error::WithLocation {
			location: input.as_slice(),
			error: error::Error::Nom(nom::error::ErrorKind::Eof),
		}))
}

fn span(input: &[Token]) -> ParseResult<'_, crate::Span> {
	token(input).map(|(rest, matched)| (rest, matched.span))
}

/*
fn template(input: &[Token]) -> ParseResult<'_, cst::T> {
	todo!()
}

*/
