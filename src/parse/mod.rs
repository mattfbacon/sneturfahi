use nom::branch::alt;
use nom::combinator::{all_consuming, cut, map, opt};
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
			selbri,
			args: args.into_boxed_slice(),
			num_args_before_selbri,
		},
	))
}

fn selbri(input: &[Token]) -> ParseResult<'_, cst::Selbri> {
	map(
		many1(separated(
			separated(
				selbri_component_outer,
				|input| tuple((selbri_connective, selmaho!(Bo)))(input),
				false,
			),
			selbri_connective,
			false,
		)),
		|components| cst::Selbri { components },
	)(input)
}

fn selbri_component_outer(input: &[Token]) -> ParseResult<'_, cst::SelbriComponentOuter> {
	alt((
		map(
			tuple((
				selmaho!(Guha),
				cut(selbri),
				selmaho!(Gi),
				selbri_component_outer,
			)),
			|(guha, first, gi, second)| cst::SelbriComponentOuter::Connected {
				guha,
				first: Box::new(first),
				gi,
				second: Box::new(second),
			},
		),
		map(
			separated(selbri_component, selmaho!(Bo), true),
			cst::SelbriComponentOuter::NotConnected,
		),
	))(input)
}

fn selbri_connective(input: &[Token]) -> ParseResult<'_, cst::SelbriConnective> {
	map(
		tuple((
			opt(selmaho!(Na)),
			opt(selmaho!(Se)),
			selbri_connective_word,
			opt(selmaho!(Nai)),
		)),
		|(na, se, word, nai)| cst::SelbriConnective { na, se, word, nai },
	)(input)
}

fn selbri_connective_word(input: &[Token]) -> ParseResult<'_, cst::SelbriConnectiveWord> {
	alt((
		map(selmaho!(Ja), cst::SelbriConnectiveWord::Ja),
		map(selmaho!(Joi), cst::SelbriConnectiveWord::Joi),
	))(input)
}

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

fn before_selbri_component(input: &[Token]) -> ParseResult<'_, cst::BeforeSelbriComponent> {
	alt((
		map(selmaho!(Jai), cst::BeforeSelbriComponent::Jai),
		map(selmaho!(Nahe), cst::BeforeSelbriComponent::Nahe),
		map(selmaho!(Se), cst::BeforeSelbriComponent::Se),
	))(input)
}

fn selbri_word(input: &[Token]) -> ParseResult<'_, cst::SelbriWord> {
	alt((
		map(
			tuple((selmaho!(Ke), cut(selbri), opt(selmaho!(Kehe)))),
			|(ke, group, kehe)| cst::SelbriWord::GroupedTanru {
				ke,
				group: Box::new(group),
				kehe,
			},
		),
		map(selmaho!(Gismu), cst::SelbriWord::Gismu),
		map(selmaho!(Lujvo), cst::SelbriWord::Lujvo),
		map(selmaho!(Fuhivla), cst::SelbriWord::Fuhivla),
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
	alt((map(tag, cst::Arg::Tag), map(sumti, cst::Arg::Sumti)))(input)
}

fn tag(input: &[Token]) -> ParseResult<'_, cst::Tag> {
	map(tuple((tag_word, tag_value)), |(word, value)| cst::Tag {
		word,
		value,
	})(input)
}

fn tag_word(input: &[Token]) -> ParseResult<'_, cst::TagWord> {
	alt((
		map(selmaho!(Bai), cst::TagWord::Bai),
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
		separated(sumti_component, sumti_connective, false),
		|inner| cst::Sumti { inner },
	)(input)
}

fn sumti_component(input: &[Token]) -> ParseResult<'_, cst::SumtiComponent> {
	alt((
		map(selmaho!(Koha), cst::SumtiComponent::Koha),
		map(le_sumti, cst::SumtiComponent::Le),
		map(la_sumti, cst::SumtiComponent::La),
		map(zo_sumti, cst::SumtiComponent::Zo),
		map(zoi_sumti, cst::SumtiComponent::Zoi),
	))(input)
}

fn le_sumti(input: &[Token]) -> ParseResult<'_, cst::LeSumti> {
	map(
		tuple((selmaho!(Le), cut(selbri), opt(selmaho!(Ku)))),
		|(le, selbri, _ku)| cst::LeSumti { le, selbri },
	)(input)
}

fn la_sumti(input: &[Token]) -> ParseResult<'_, cst::LaSumti> {
	map(tuple((selmaho!(La), cut(la_sumti_inner))), |(la, inner)| {
		cst::LaSumti { la, inner }
	})(input)
}

fn la_sumti_inner(input: &[Token]) -> ParseResult<'_, cst::LaSumtiInner> {
	alt((
		map(many1(selmaho!(Cmevla)), cst::LaSumtiInner::Cmevla),
		map(tuple((selbri, opt(selmaho!(Ku)))), |(selbri, _ku)| {
			cst::LaSumtiInner::Selbri(selbri)
		}),
	))(input)
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
