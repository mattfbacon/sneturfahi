#![allow(missing_docs, missing_copy_implementations)]

// https://raw.githubusercontent.com/lojban/camxes-py/master/camxes_py/parsers/camxes_ilmen.peg

use macros::Parse;

use crate::lex::{Selmaho, Token};
use crate::span::Span;

pub mod connectives;
pub use connectives::{Gek, Gihek, Gik, Guhek, Interval, Joik, JoikEk, JoikJek};

pub mod mekso;
pub use mekso::{Expression as Mekso, Operator as MeksoOperator};

pub(super) trait Parse: Sized {
	fn parse(input: &[Token]) -> super::ParseResult<'_, Self>;
}

impl<T: Parse> Parse for Option<T> {
	fn parse(input: &[Token]) -> super::ParseResult<'_, Self> {
		nom::combinator::opt(Parse::parse)(input)
	}
}

impl<T: Parse> Parse for Box<T> {
	fn parse(input: &[Token]) -> super::ParseResult<'_, Self> {
		nom::combinator::map(Parse::parse, Box::new)(input)
	}
}

impl Parse for Token {
	fn parse(input: &[Token]) -> super::ParseResult<'_, Self> {
		let mut input = input.iter();
		input
			.next()
			.map(|&token| (input.as_slice(), token))
			.ok_or(nom::Err::Error(super::error::WithLocation {
				location: input.as_slice(),
				error: super::error::Error::Nom(nom::error::ErrorKind::Eof),
			}))
	}
}

impl Parse for Span {
	fn parse(input: &[Token]) -> super::ParseResult<'_, Self> {
		Token::parse(input).map(|(rest, matched)| (rest, matched.span))
	}
}

macro_rules! tuple_impls {
	// base case
	() => {};
	(@single $($idents:ident),*) => {
		impl<$($idents: Parse),*> Parse for ($($idents,)*) {
			fn parse(input: &[Token]) -> super::ParseResult<'_, Self> {
				nom::sequence::tuple(($(<$idents as Parse>::parse,)*))(input)
			}
		}
	};
	($first:ident $(, $idents:ident)*) => {
		tuple_impls!(@single $first $(, $idents)*);
		tuple_impls!($($idents),*);
	};
}

tuple_impls![T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15];

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("wrong selmaho: expected {expected}, got {got}")]
pub(super) struct WrongSelmaho {
	expected: Selmaho,
	got: Selmaho,
}

pub(super) trait SelmahoTypeRaw:
	TryFrom<Token, Error = super::Error> + TryFrom<Option<Token>, Error = super::Error>
{
}

macro_rules! token_types {
	(@raw $name:ident) => {
		#[derive(Debug)]
		pub struct $name {
			pub experimental: bool,
			pub span: Span,
		}

		impl TryFrom<Token> for $name {
			type Error = super::Error;

			fn try_from(token: Token) -> Result<Self, super::Error> {
				if token.selmaho == Selmaho::$name {
					Ok(Self {
						experimental: token.experimental,
						span: token.span,
					})
				} else {
					Err(super::Error::ExpectedGot {
						expected: (&[Selmaho::$name] as &[Selmaho]).into(),
						got: Some(token),
					})
				}
			}
		}

		impl TryFrom<Option<Token>> for $name {
			type Error = super::Error;

			fn try_from(token: Option<Token>) -> Result<Self, super::Error> {
				token
					.ok_or(super::Error::ExpectedGot {
						expected: (&[Selmaho::$name] as &[Selmaho]).into(),
						got: None,
					})
					.and_then(Self::try_from)
			}
		}

		impl SelmahoTypeRaw for $name {}

		impl Parse for $name {
			fn parse<'a>(input: &'a [Token]) -> super::ParseResult<'a, Self> {
				super::selmaho_raw::<$name>(input)
			}
		}
	};
	(@no_indicators $name:ident) => {
		#[derive(Debug)]
		pub struct $name {
			pub bahe: Box<[Bahe]>,
			pub experimental: bool,
			pub span: Span,
		}

		impl TryFrom<Token> for $name {
			type Error = super::Error;

			fn try_from(token: Token) -> Result<Self, super::Error> {
				if token.selmaho == Selmaho::$name {
					Ok(Self {
						bahe: Box::new([]),
						experimental: token.experimental,
						span: token.span,
					})
				} else {
					Err(super::Error::ExpectedGot {
						expected: (&[Selmaho::$name] as &[Selmaho]).into(),
						got: Some(token),
					})
				}
			}
		}

		impl TryFrom<Option<Token>> for $name {
			type Error = super::Error;

			fn try_from(token: Option<Token>) -> Result<Self, super::Error> {
				token.ok_or(super::Error::ExpectedGot { expected: (&[Selmaho::$name] as &[Selmaho]).into(), got: None }).and_then(Self::try_from)
			}
		}
		impl SelmahoTypeRaw for $name {}

		impl Parse for $name {
			fn parse<'a>(input: &'a [Token]) -> super::ParseResult<'a, Self> {
				let (input, bahe) = nom::Parser::parse(&mut super::many0(Bahe::parse), input)?;
				let (input, mut matched) = super::selmaho_raw::<Self>(input)?;
				matched.bahe = bahe;
				Ok((input, matched))
			}
		}
	};
	(@single $name:ident) => {
		#[derive(Debug)]
		pub struct $name {
			pub bahe: Box<[Bahe]>,
			pub experimental: bool,
			pub span: Span,
			pub indicators: Option<Box<Indicators>>,
		}

		impl TryFrom<Token> for $name {
			type Error = super::Error;

			fn try_from(token: Token) -> Result<Self, super::Error> {
				if token.selmaho == Selmaho::$name {
					Ok(Self {
						bahe: Box::new([]),
						experimental: token.experimental,
						span: token.span,
						indicators: None,
					})
				} else {
					Err(super::Error::ExpectedGot {
						expected: (&[Selmaho::$name] as &[Selmaho]).into(),
						got: Some(token),
					})
				}
			}
		}

		impl TryFrom<Option<Token>> for $name {
			type Error = super::Error;

			fn try_from(token: Option<Token>) -> Result<Self, super::Error> {
				token.ok_or(super::Error::ExpectedGot { expected: (&[Selmaho::$name] as &[Selmaho]).into(), got: None }).and_then(Self::try_from)
			}
		}
		impl SelmahoTypeRaw for $name {}

		impl Parse for $name {
			fn parse<'a>(input: &'a [Token]) -> super::ParseResult<'a, Self> {
				let (input, bahe) = nom::Parser::parse(&mut super::many0(super::selmaho_raw::<Bahe>), input)?;
				let (input, mut matched) = super::selmaho_raw::<Self>(input)?;
				let (input, indicators) = <Option<Box<Indicators>>>::parse(input)?;
				matched.bahe = bahe;
				matched.indicators = indicators;
				Ok((input, matched))
			}
		}
	};
	($(,)?) => {};
	($(,)? $name:ident $($rest:tt)*) => {
		token_types!(@single $name);
		token_types!($($rest)*);
	};
	($(,)? #[no_indicators] $name:ident $($rest:tt)*) => {
		token_types!(@no_indicators $name);
		token_types!($($rest)*);
	};
	($(,)? #[raw] $name:ident $($rest:tt)*) => {
		token_types!(@raw $name);
		token_types!($($rest)*);
	};
}

token_types! {
	A,
	#[raw] Bahe,
	Bai,
	Be,
	Beho,
	Bei,
	Bihe,
	Bihi,
	Bo,
	Boi,
	#[raw] Bu,
	By,
	Caha,
	Cai,
	Cehe,
	Cei,
	Cmevla,
	Co,
	Coi,
	Giha,
	Cu,
	Cuhe,
	Daho,
	Dohu,
	Doi,
	Tuhe,
	Tuhu,
	Fa,
	Faha,
	Faho,
	Fehe,
	Fehu,
	Fiho,
	Foi,
	Fuha,
	#[no_indicators] Fuhe,
	Fuhivla,
	Fuho,
	Ga,
	Gaho,
	Gehu,
	Gi,
	Gismu,
	Goha,
	Goi,
	Niho,
	Guha,
	I,
	Ja,
	Jai,
	Johi,
	Joi,
	Ke,
	Kehe,
	Kei,
	Ki,
	Koha,
	Ku,
	Kuhe,
	Kuho,
	La,
	Lahe,
	Lau,
	Le,
	Lehu,
	Li,
	Lihu,
	Loho,
	#[raw] Lohu,
	#[raw] Lu,
	Luhu,
	Lujvo,
	Maho,
	Mai,
	Me,
	Mehu,
	Mohe,
	Mohi,
	Moi,
	Na,
	Nahe,
	Nahu,
	Nai,
	Nihe,
	Noi,
	Nu,
	Nuha,
	Nuhi,
	Nuhu,
	Pa,
	Pehe,
	Peho,
	Pu,
	Raho,
	Roi,
	Se,
	Sehu,
	Sei,
	Soi,
	Tahe,
	Tehu,
	Tei,
	To,
	Toi,
	Ui,
	Va,
	Vau,
	Veha,
	Veho,
	Vei,
	Viha,
	Vuho,
	Vuhu,
	Xi,
	Y,
	Zaho,
	Zeha,
	Zi,
	Zihe,
	#[raw] Zo,
	Zohu,
	#[raw] Zoi,
}

#[derive(Parse)]
pub struct Separated<Item, Separator> {
	pub first: Box<Item>,
	#[parse(with = "super::many0(super::tuple((Separator::parse, Item::parse)))")]
	pub rest: Box<[(Separator, Item)]>,
}

// print as a single list with the separators interleaved. obviously this would not be valid rust, but it cuts down indentation.
impl<Item: std::fmt::Debug, Separator: std::fmt::Debug> std::fmt::Debug
	for Separated<Item, Separator>
{
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		formatter.write_str("Separated ")?;
		let mut list = formatter.debug_list();
		list.entry(&self.first);
		for (separator, item) in self.rest.iter() {
			list.entry(separator);
			list.entry(item);
		}
		list.finish()?;
		Ok(())
	}
}

#[derive(Debug, Parse)]
pub enum EitherOrBoth<L, R> {
	Right(R),
	Both(L, R),
	Left(L),
}

#[derive(Debug, Parse)]
pub struct WithFree<Inner> {
	pub inner: Inner,
	pub frees: Frees,
}

#[derive(Debug, Parse)]
pub struct Frees(#[parse(with = "super::many0(Parse::parse)")] pub Box<[Free]>);

pub type Root = Text;

#[derive(Debug, Parse)]
pub struct Text {
	pub initial_indicators: Option<Indicators>,
	pub initial_frees: Frees,
	pub initial_paragraph_separator: Option<ParagraphSeparator>,
	pub paragraphs: Option<Paragraphs>,
	pub faho: Option<Faho>,
}

#[derive(Debug, Parse)]
pub struct Paragraphs(Separated<Paragraph, ParagraphSeparator>);

#[derive(Debug, Parse)]
pub struct Paragraph {
	pub initial_sentence_separator: Option<SentenceSeparator>,
	pub sentences: ParagraphItems,
}

pub type ParagraphItems = Separated<ParagraphItem, SentenceSeparator>;

#[derive(Debug, Parse)]
pub enum ParagraphItem {
	#[parse(must_consume)]
	Sentences(Sentences1),
	Fragment(Box<Fragment>),
	Empty(),
}

#[derive(Debug, Parse)]
pub enum Fragment {
	// answer to ek connective question
	Ek(WithFree<JoikEk>),
	// answer to gihek connective question
	Gihek(WithFree<Gihek>),
	// answer to number question
	// this is Quantifier rather than something that accepts MiscNumber because a lerfu string that starts with a letteral can be parsed as a sumti instead
	Number(Quantifier),
	// answer to negation question?
	Na(Na, #[parse(not = "Ja")] Frees),
}

#[derive(Debug, Parse)]
pub struct Sentences1(
	#[parse(with = "super::many0(Parse::parse)")] pub Box<[Prenex]>,
	pub Separated<Sentences2, ConnectedSentenceSeparator>,
);
pub type Sentences2 = Separated<Sentences3, CloseSentenceSeparator>;

#[derive(Debug, Parse)]
pub enum Sentences3 {
	Grouped(
		Option<TagWords>,
		WithFree<Tuhe>,
		Paragraphs,
		Option<Tuhu>,
		Frees,
	),
	Single(Box<Sentence>),
}

#[derive(Debug, Parse)]
pub struct ParagraphSeparator(
	#[parse(with = "super::many1(Parse::parse)")] Box<[Niho]>,
	Frees,
);

#[derive(Debug, Parse)]
pub struct SentenceSeparator(pub I, #[parse(not = "Bu")] pub Frees);

#[derive(Debug, Parse)]
pub struct ConnectedSentenceSeparator(pub I, pub JoikJek, pub Frees);

#[derive(Debug, Parse)]
pub struct CloseSentenceSeparator(
	pub I,
	pub Option<JoikJek>,
	pub Option<TagWords>,
	pub Bo,
	pub Frees,
);

#[derive(Debug, Parse)]
pub struct Sentence {
	pub before_args: Args,
	pub tail: Option<SentenceTail>,
}

#[derive(Debug, Parse)]
pub struct SentenceTail(pub Option<Cu>, pub Frees, pub SentenceTail1);

#[derive(Debug, Parse)]
pub struct SentenceTail1(pub SentenceTail2, pub Option<SentenceTail1After>);

#[derive(Debug, Parse)]
pub struct SentenceTail1After(
	Gihek,
	Option<TagWords>,
	Ke,
	Frees,
	Box<SentenceTail1>,
	Option<Kehe>,
	Frees,
	TailArgs,
);

#[derive(Debug, Parse)]
pub struct SentenceTail2(pub Box<SentenceTail3>, pub Option<SentenceTail2After>);

#[derive(Debug, Parse)]
pub struct SentenceTail2Connective(
	pub Gihek,
	// `gi'e ke ...` must be parsed as SentenceTail1After, not a parenthesized tanru
	#[parse(not = "(Option<TagWords>, Ke)")] pub Frees,
);

#[derive(Debug, Parse)]
pub struct SentenceTail2After(
	#[parse(with = "super::many1(Parse::parse)")] pub Box<[(SentenceTail2Connective, SentenceTail3)]>,
	pub TailArgs,
);

#[derive(Debug, Parse)]
pub struct SentenceTail3(pub Box<SentenceTail4>, pub Option<SentenceTail3After>);

pub type SentenceTail3Connective = (Gihek, Option<TagWords>, Bo, Frees);

#[derive(Debug, Parse)]
pub struct SentenceTail3After(
	#[parse(with = "super::many1(Parse::parse)")] Box<[(SentenceTail3Connective, SentenceTail4)]>,
	TailArgs,
);

#[derive(Debug, Parse)]
pub enum SentenceTail4 {
	Single(Selbri, TailArgs),
	Parenthesized(
		#[parse(with = "super::many0(Parse::parse)")] Box<[WithFree<Na>]>,
		Option<TagWords>,
		WithFree<Ke>,
		Box<GekSentence>,
		Option<Kehe>,
		Frees,
	),
	Connected(Box<GekSentence>),
}

#[derive(Debug, Parse)]
pub struct GekSentence(
	#[parse(with = "super::many0(Parse::parse)")] pub Box<[WithFree<Na>]>,
	pub Gek,
	pub Subsentence,
	pub Gik,
	pub Subsentence,
	pub TailArgs,
);

#[derive(Debug, Parse)]
pub struct Subsentence(
	#[parse(with = "super::many0(Parse::parse)")] pub Box<[Prenex]>,
	Args,
	SentenceTail,
);

#[derive(Debug, Parse)]
pub struct Args(#[parse(with = "super::many0(Parse::parse)")] pub Box<[Arg]>);

#[derive(Debug, Parse)]
pub struct TailArgs(pub Args, pub Option<Vau>, pub Frees);

#[derive(Debug, Parse)]
pub struct Prenex {
	pub terms: Args,
	pub zohu: Zohu,
	pub frees: Frees,
}

#[derive(Debug, Parse)]
pub struct Arg(
	Arg1,
	#[parse(with = "super::many0(Parse::parse)")] Box<[SumtiLikeConnectedPost<Arg1, Self>]>,
);

pub type Arg1 = Separated<Arg2, PeheConnective>;

#[derive(Debug, Parse)]
pub struct PeheConnective(WithFree<Pehe>, JoikJek);

pub type Arg2 = Separated<Arg3, WithFree<Cehe>>;

#[derive(Debug, Parse)]
pub enum Arg3 {
	Tag(Tag),
	Sumti {
		fa: Option<WithFree<Fa>>,
		sumti: Sumti,
	},
	Naku(Na, Ku, Frees),
	Termset(Box<Termset>),
}

#[derive(Debug, Parse)]
pub enum Termset {
	Gek(Gek, Args, Gik, Args),
	NuhiGi(
		WithFree<Nuhi>,
		Gek,
		Args,
		Option<Nuhu>,
		Frees,
		Gik,
		Args,
		Option<Nuhu>,
		Frees,
	),
	Nuhi(WithFree<Nuhi>, Args, Option<Nuhu>, Frees),
}

#[derive(Debug, Parse)]
pub struct Selbri {
	#[parse(with = "super::many0(Parse::parse)")]
	pub before: Box<[SelbriBefore]>,
	// all other `Separated` will use `false` for `should_cut`
	pub components: Selbri1,
}

#[derive(Debug, Parse)]
pub enum SelbriBefore {
	Na(WithFree<Na>),
	Tag(TagWords),
}

#[derive(Debug, Parse)]
#[repr(transparent)]
pub struct Selbri1(Separated<Selbri2, WithFree<Co>>);

#[derive(Debug, Parse)]
#[repr(transparent)]
pub struct Selbri2(#[parse(with = "super::many1(Parse::parse)")] Box<[Selbri3]>);

#[derive(Debug, Parse)]
pub struct Selbri3(
	pub Selbri4,
	#[parse(with = "super::many0(Parse::parse)")] pub Box<[Selbri3ConnectedPost]>,
);

pub type Selbri3ConnectedPost = SelbriLikeConnectedPost<Selbri4, Selbri2>;

#[derive(Debug, Parse)]
pub enum SelbriLikeConnectedPost<Normal, Parenthesized> {
	Normal(JoikJek, Normal),
	Parenthesized(
		Joik,
		Option<TagWords>,
		WithFree<Ke>,
		Parenthesized,
		Option<Kehe>,
		Frees,
	),
}

pub type Selbri4 = Separated<Selbri5, (JoikJek, Option<TagWords>, Bo, Frees)>;

#[derive(Debug, Parse)]
pub struct Selbri5(
	#[parse(with = "super::many0(Parse::parse)")] pub Box<[NaheGuhekTGik<Selbri>]>,
	pub Selbri6,
);

#[derive(Debug, Parse)]
pub struct NaheGuhekTGik<T>(pub Option<Nahe>, pub Frees, pub Guhek, pub T, pub Gik);

#[derive(Debug, Parse)]
pub struct Selbri6(pub Separated<TanruUnit, WithFree<Bo>>);

#[derive(Debug, Parse)]
#[repr(transparent)]
pub struct TanruUnit(pub Separated<TanruUnit1, WithFree<Cei>>);

#[derive(Debug, Parse)]
pub struct TanruUnit1 {
	#[parse(with = "super::many0(Parse::parse)")]
	pub before: Box<[BeforeTanruUnit]>,
	pub inner: TanruUnit2,
	pub bound_arguments: Option<BoundArguments>,
}

#[derive(Debug, Parse)]
pub enum BeforeTanruUnit {
	Jai {
		jai: WithFree<Jai>,
		tag: Option<TagWords>,
	},
	Nahe(WithFree<Nahe>),
	Se(WithFree<Se>),
}

#[derive(Debug, Parse)]
pub struct BoundArguments {
	pub be: WithFree<Be>,
	pub args: Separated<Arg, WithFree<Bei>>,
	pub beho: Option<Beho>,
	pub frees: Frees,
}

#[derive(Debug, Parse)]
pub enum TanruUnit2 {
	GroupedTanru {
		ke: WithFree<Ke>,
		group: Selbri2, /* not `Selbri` because ke-ke'e groupings can't encompass co (CLL 5.8) nor tense, modal, and negation cmavo (CLL 5.13). `Selbri2` is inside co groupings (`Selbri1`) and na/tags (`Selbri`). */
		kehe: Option<Kehe>,
		frees: Frees,
	},
	Gismu(WithFree<Gismu>),
	Lujvo(WithFree<Lujvo>),
	Fuhivla(WithFree<Fuhivla>),
	Goha {
		goha: Goha,
		raho: Option<Raho>,
		frees: Frees,
	},
	Moi(MiscNumbers, Moi, Frees),
	Me {
		me: WithFree<Me>,
		inner: Sumti,
		mehu: Option<Mehu>,
		frees: Frees,
		moi: Option<WithFree<Moi>>,
	},
	Nu {
		nus: Separated<(Nu, Option<Nai>, Frees), JoikJek>,
		inner: Box<Subsentence>,
		kei: Option<Kei>,
		frees: Frees,
	},
	Nuha {
		nuha: WithFree<Nuha>,
		operator: MeksoOperator,
	},
}

#[derive(Debug, Parse)]
pub struct Tag {
	pub words: TagWords,
	// e.g., "ri'agi broda gi brode" or "ri'agi ko'a gi ko'e". this rule would consume `ri'a` with no argument and leave just a `gi`.
	#[parse(not = "Gik", not = "Ke")]
	pub value: Option<TagValue>,
}

#[derive(Debug, Parse)]
pub enum TagValue {
	Ku(WithFree<Ku>),
	Sumti(Sumti),
}

pub type TagWords = Separated<TagWord, JoikJek>;

#[derive(Debug, Parse)]
pub enum TagWord {
	Bai {
		nahe: Option<Nahe>,
		se: Option<Se>,
		bai: Bai,
		nai: Option<Nai>,
		ki: Option<Ki>,
		frees: Frees,
	},
	TimeSpaceCaha {
		nahe: Option<Nahe>,
		#[parse(with = "super::many1(Parse::parse)")]
		inner: Box<[TimeSpaceCaha]>,
		ki: Option<Ki>,
		frees: Frees,
	},
	Ki(Ki, Frees),
	Cuhe(Cuhe, Frees),
	Converted(WithFree<Fiho>, Selbri, Option<Fehu>, Frees),
}

#[derive(Debug, Parse)]
pub enum TimeSpaceCaha {
	Time(Time),
	Space(Box<Space>),
	Caha(Caha),
}

#[derive(Debug, Parse)]
#[parse(postcond("|time| time.zi.is_some() || !time.offset.is_empty() || time.duration.is_some() || !time.properties.is_empty()"))]
pub struct Time {
	pub zi: Option<Zi>,
	#[parse(with = "super::many0(Parse::parse)")]
	pub offset: Box<[TimeOffset]>,
	pub duration: Option<TimeDuration>,
	#[parse(with = "super::many0(Parse::parse)")]
	pub properties: Box<[TimeIntervalProperty]>,
}

#[derive(Debug, Parse)]
pub struct TimeOffset {
	pub pu: Pu,
	pub nai: Option<Nai>,
	pub zi: Option<Zi>,
}

#[derive(Debug, Parse)]
pub struct TimeDuration {
	pub zeha: Zeha,
	/// see CLL 10.5, specifically examples 10.26 through 10.29
	pub anchor: Option<(Pu, Option<Nai>)>,
}

#[derive(Debug, Parse)]
pub enum IntervalProperty {
	Roi(Number, Roi, Option<Nai>),
	Tahe(Tahe, Option<Nai>),
	Zaho(Zaho, Option<Nai>),
}

pub type TimeIntervalProperty = IntervalProperty;

#[derive(Debug, Parse)]
#[parse(postcond("|space| space.va.is_some() || !space.offset.is_empty() || space.interval.is_some() || space.motion.is_some()"))]
pub struct Space {
	pub va: Option<Va>,
	#[parse(with = "super::many0(Parse::parse)")]
	pub offset: Box<[SpaceOffset]>,
	pub interval: Option<SpaceInterval>,
	pub motion: Option<SpaceMotion>,
}

#[derive(Debug, Parse)]
pub struct SpaceOffset(Faha, Option<Nai>, Option<Va>);

#[derive(Debug, Parse)]
pub enum SpaceInterval {
	Interval {
		interval: EitherOrBoth<Veha, Viha>,
		direction: Option<(Faha, Option<Nai>)>,
		#[parse(with = "super::many0(Parse::parse)")]
		properties: Box<[SpaceIntervalProperty]>,
	},
	Properties(#[parse(with = "super::many1(Parse::parse)")] Box<[SpaceIntervalProperty]>),
}

#[derive(Debug, Parse)]
pub struct SpaceIntervalProperty(Fehe, IntervalProperty);

#[derive(Debug, Parse)]
pub struct SpaceMotion {
	pub mohi: Mohi,
	pub offset: SpaceOffset,
}

#[derive(Debug, Parse)]
pub struct Sumti {
	pub inner: Sumti1,
	pub vuho_relative: Option<VuhoRelative>,
}

#[derive(Debug, Parse)]
pub struct Sumti1(
	pub Sumti2,
	#[parse(with = "super::many0(Parse::parse)")] pub Box<[SumtiLikeConnectedPost<Sumti2, Sumti>]>,
);

#[derive(Debug, Parse)]
pub enum SumtiLikeConnectedPost<Normal, Parenthesized> {
	Normal(JoikEk, Normal),
	Grouped(
		JoikEk,
		Option<TagWords>,
		WithFree<Ke>,
		Parenthesized,
		Option<Kehe>,
		Frees,
	),
}

pub type Sumti2 = Separated<Sumti3, (JoikEk, Option<TagWords>, Bo, Frees)>;

#[derive(Debug, Parse)]
pub struct VuhoRelative {
	pub vuho: WithFree<Vuho>,
	pub relative_clauses: RelativeClauses,
}

#[derive(Debug, Parse)]
pub struct RelativeClauses(Separated<RelativeClause, WithFree<Zihe>>);

#[derive(Debug, Parse)]
pub enum RelativeClause {
	Goi(GoiRelativeClause),
	Noi(NoiRelativeClause),
}

#[derive(Debug, Parse)]
pub struct GoiRelativeClause {
	pub goi: WithFree<Goi>,
	/// typical usage would match Arg::Sumti, but Arg::Tag is possible as well, such as in `la salis nesemau la betis cu se prami mi`
	pub inner: Arg,
	pub gehu: Option<Gehu>,
	pub frees: Frees,
}

#[derive(Debug, Parse)]
pub struct NoiRelativeClause {
	pub noi: WithFree<Noi>,
	pub inner: Box<Subsentence>,
	pub kuho: Option<Kuho>,
	pub frees: Frees,
}

#[derive(Debug, Parse)]
pub struct Sumti3(
	#[parse(with = "super::many0(Parse::parse)")] pub Box<[Sumti3ConnectedPre]>,
	Sumti4,
);

#[derive(Debug, Parse)]
pub struct Sumti3ConnectedPre(pub Gek, pub Sumti, pub Gik);

#[derive(Debug, Parse)]
pub enum Sumti4 {
	Normal {
		quantifier: Option<Quantifier>,
		inner: Box<SumtiComponent>,
		relative_clauses: Option<RelativeClauses>,
	},
	SelbriShorthand {
		quantifier: Quantifier,
		inner: Selbri,
		ku: Option<Ku>,
		frees: Frees,
		relative_clauses: Option<RelativeClauses>,
	},
}

#[derive(Debug, Parse)] // similar to part of `mekso::Operand3`
pub enum Quantifier {
	Mekso(WithFree<Vei>, Mekso, Option<Veho>, Frees),
	Number(Number, #[parse(not = "Moi")] Option<Boi>, Frees),
}

#[derive(Debug, Parse)]
pub struct Number {
	pub first: Pa,
	#[parse(with = "super::many0(Parse::parse)")]
	pub rest: Box<[NumberRest]>,
}

#[derive(Debug, Parse)]
pub enum NumberRest {
	Pa(Pa),
	Lerfu(LerfuWord),
}

#[derive(Debug, Parse)]
pub struct LerfuString {
	pub first: LerfuWord,
	#[parse(with = "super::many0(Parse::parse)")]
	pub rest: Box<[NumberRest]>,
}

#[derive(Debug, Parse)]
pub struct MiscNumbers(#[parse(with = "super::many1(Parse::parse)")] Box<[NumberRest]>);

#[derive(Debug, Parse)]
pub enum LerfuWord {
	Lerfu(Lerfu),
	Lau {
		lau: Lau,
		lerfu: Lerfu,
	},
	Tei {
		tei: Tei,
		inner: Box<LerfuString>, // recursion avoided here
		#[cut]
		foi: Foi,
	},
}

#[derive(Debug, Parse)]
pub enum Lerfu {
	Bu(
		Option<Bahe>,
		BuLerfu,
		#[parse(with = "super::many1(Parse::parse)")] Box<[Bu]>,
		#[parse(with = "super::many0(Parse::parse)")] Box<[Indicators]>,
	),
	By(By),
}

#[derive(Debug, Parse)]
#[parse(
	postcond = "|Self(token)| !matches!(token.selmaho, Selmaho::Bu | Selmaho::Zei | Selmaho::Si | Selmaho::Su | Selmaho::Sa | Selmaho::Faho)"
)]
pub struct BuLerfu(Token);

pub type SumtiComponent = WithFree<SumtiComponent1>;

#[derive(Debug, Parse)]
pub enum SumtiComponent1 {
	Koha(Koha),
	Gadri(Box<GadriSumti>),
	La(LaSumti),
	Lohu(LohuSumti),
	Lu(LuSumti),
	Modified(ModifiedSumti),
	LerfuString(LerfuString, #[parse(not = "Moi")] Option<Boi>),
	Zo(ZoSumti),
	Zoi(ZoiSumti),
	Li(WithFree<Li>, Mekso, Option<Loho>),
}

#[derive(Debug)]
pub struct LohuSumti {
	pub lohu: Lohu,
	pub inner: Box<[Token]>,
	pub lehu: Lehu,
}

impl Parse for LohuSumti {
	fn parse(input: &[Token]) -> super::ParseResult<'_, Self> {
		nom::combinator::map(
			nom::sequence::tuple((
				Parse::parse,
				nom::combinator::cut(nom::multi::many_till(Parse::parse, Parse::parse)),
			)),
			|(lohu, (inner, lehu))| Self {
				lohu,
				inner: inner.into_boxed_slice(),
				lehu,
			},
		)(input)
	}
}

#[derive(Debug, Parse)]
pub struct LuSumti {
	pub lu: Lu,
	pub text: Text,
	pub lihu: Option<Lihu>,
}

#[derive(Debug, Parse)]
pub struct ModifiedSumti {
	pub modifier: WithFree<SumtiModifier>,
	pub relative_clauses: Option<RelativeClauses>,
	pub sumti: Sumti,
	pub luhu: Option<Luhu>,
}

#[derive(Debug, Parse)]
pub enum SumtiModifier {
	Lahe(Lahe),
	NaheBo(Nahe, Bo),
}

#[derive(Debug, Parse)]
pub struct GadriSumti {
	pub gadri: WithFree<Gadri>,
	pub pre: GadriSumtiPre,
	pub contents: GadriSumtiContents,
	pub ku: Option<Ku>,
}

#[derive(Debug, Parse)]
pub enum Gadri {
	Le(Le),
	La(La),
}

#[derive(Debug, Parse)]
pub enum GadriSumtiPre {
	// order is important here
	Simple {
		pe_shorthand: Option<Box<SumtiComponent>>, // recursion avoided here
		relative_clauses: Option<RelativeClauses>,
	},
	Relative {
		relative_clauses: RelativeClauses,
	},
	FullPre {
		pe_shorthand: Box<Sumti>,
	},
}

#[derive(Debug, Parse)]
pub enum GadriSumtiContents {
	Selbri(Option<Quantifier>, Selbri, Option<RelativeClauses>),
	Sumti(Quantifier, Sumti),
}

#[derive(Debug, Parse)]
pub struct LaSumti {
	pub la: La,
	#[parse(with = "super::many1(Parse::parse)")]
	pub inner: Box<[Cmevla]>,
}

#[derive(Debug, Parse)]
pub struct ZoSumti {
	pub zo: Zo,
	pub quoted: Token,
}

#[derive(Debug, Parse)]
pub struct ZoiSumti {
	pub zoi: Zoi,
	pub starting_delimiter: Span,
	pub text: Span,
	pub ending_delimiter: Span,
}

#[derive(Debug, Parse)]
pub enum Free {
	Sei(WithFree<Sei>, Args, Option<SeiTail>, Option<WithFree<Sehu>>),
	Soi(WithFree<Soi>, Box<(Sumti, Option<Sumti>)>, Option<Sehu>),
	Vocative(Vocative),
	Mai(MiscNumbers, Mai),
	To(To, Text, Option<Toi>),
	Xi(Subscript),
}

#[derive(Debug, Parse)]
pub struct Vocative(
	pub VocativeWords,
	pub Option<RelativeClauses>,
	pub VocativeValue,
	pub Option<RelativeClauses>,
	pub Option<Dohu>,
);

#[derive(Debug, Parse)]
pub struct SeiTail(pub Option<Cu>, pub Frees, pub Selbri);

#[derive(Debug, Parse)]
pub enum VocativeWords {
	Coi(
		#[parse(with = "super::many1(Parse::parse)")] Box<[(Coi, Option<Nai>)]>,
		Option<Doi>,
	),
	Doi(Doi),
}

#[derive(Debug, Parse)]
pub enum VocativeValue {
	Selbri(Box<Selbri>),
	Cmevla(#[parse(with = "super::many1(Parse::parse)")] Box<[Cmevla]>),
	Sumti(Option<Box<Sumti>>),
}

#[derive(Debug, Parse)]
pub struct Subscript(pub WithFree<Xi>, pub WithFree<SubscriptValue>);

#[derive(Debug, Parse)] // similar to part of `mekso::Operand3`
pub enum SubscriptValue {
	Mekso(WithFree<Vei>, Mekso, Option<Veho>, Frees),
	Number(MiscNumbers, #[parse(not = "Moi")] Option<Boi>, Frees),
}

#[derive(Debug, Parse)]
pub struct Indicators(
	pub Option<Fuhe>,
	#[parse(with = "super::many1(Parse::parse)")] pub Box<[Indicator]>,
);

#[derive(Debug, Parse)]
#[parse(not_after = "Bu")]
pub enum Indicator {
	Ui(Ui),
	Cai(Cai),
	Nai(Nai),
	Daho(Daho),
	Fuho(Fuho),
	Y(Y),
}
