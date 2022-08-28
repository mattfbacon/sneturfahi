#![allow(missing_docs, missing_copy_implementations)]

use macros::Parse;
use nom::Parser;

use crate::lex::{Selmaho, Token};
use crate::span::Span;

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

pub(super) trait SelmahoType: SelmahoTypeRaw {
	fn set_bahe(&mut self, bahe: Box<[Bahe]>);
}

macro_rules! token_types {
	($($name:ident,)*) => {
		$(
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
			impl SelmahoType for $name {
				fn set_bahe(&mut self, bahe: Box<[Bahe]>) {
					self.bahe = bahe;
				}
			}

			impl Parse for $name {
				fn parse<'a>(input: &'a [Token]) -> super::ParseResult<'a, Self> {
					let (input, bahe) = nom::Parser::parse(&mut super::many0(super::selmaho_raw::<Bahe>), input)?;
					let (rest, mut matched) = super::selmaho_raw::<Self>(input)?;
					matched.set_bahe(bahe);
					Ok((rest, matched))
				}
			}
		)*
	}
}

#[derive(Debug)]
pub struct Bahe {
	pub experimental: bool,
	pub span: Span,
}

impl TryFrom<Token> for Bahe {
	type Error = super::Error;

	fn try_from(token: Token) -> Result<Self, super::Error> {
		if token.selmaho == Selmaho::Bahe {
			Ok(Self {
				experimental: token.experimental,
				span: token.span,
			})
		} else {
			Err(super::Error::ExpectedGot {
				expected: (&[Selmaho::Bahe] as &[Selmaho]).into(),
				got: Some(token),
			})
		}
	}
}

impl TryFrom<Option<Token>> for Bahe {
	type Error = super::Error;

	fn try_from(token: Option<Token>) -> Result<Self, super::Error> {
		token
			.ok_or(super::Error::ExpectedGot {
				expected: (&[Selmaho::Bahe] as &[Selmaho]).into(),
				got: None,
			})
			.and_then(Self::try_from)
	}
}

impl SelmahoTypeRaw for Bahe {}

token_types! {
	A,
	Bai,
	Be,
	Beho,
	Bei,
	Bo,
	Boi,
	By,
	Caha,
	Cei,
	Cmevla,
	Co,
	Cu,
	Cuhe,
	Fa,
	Faho,
	Fehu,
	Fiho,
	Foi,
	Fuhivla,
	Gehu,
	Gi,
	Gismu,
	Goha,
	Goi,
	Guha,
	I,
	Ja,
	Jai,
	Joi,
	Ke,
	Kehe,
	Kei,
	Ki,
	Koha,
	Ku,
	Kuho,
	La,
	Lahe,
	Lau,
	Le,
	Lehu,
	Roi,
	Tahe,
	Zaho,
	Li,
	Lihu,
	Loho,
	Lohu,
	Lu,
	Luhu,
	Pu,
	Lujvo,
	Me,
	Va,
	Mohi,
	Veha,
	Viha,
	Faha,
	Fehe,
	Mehu,
	Moi,
	Zeha,
	Na,
	Nahe,
	Nai,
	Noi,
	Nu,
	Nuha,
	Pa,
	Raho,
	Se,
	Tei,
	Veho,
	Vei,
	Vuho,
	Zihe,
	Zo,
	Zohu,
	Zi,
	Zoi,
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

#[derive(Debug)]
pub struct WithFree<Inner> {
	pub inner: Inner,
	pub free: Box<[Free]>,
}

// todo
#[derive(Debug)]
pub struct Free;

pub type Root = Text;

#[derive(Debug, Parse)]
pub struct Text {
	pub initial_i: Option<I>,
	#[parse(with = "super::separated(true)")]
	pub sentences: Separated<Sentence, I>,
	pub faho: Option<Faho>,
}

#[derive(Debug)]
pub struct Sentence {
	pub prenexes: Box<[Prenex]>,
	pub selbri: Option<(Option<Cu>, Selbri)>,
	pub args: Box<[Arg]>,
	/// How many of `args` were before `selbri`.
	///
	/// Will be equal to `args.len()` if there is no selbri.
	pub num_args_before_selbri: usize,
}

impl Parse for Sentence {
	fn parse(mut input: &[Token]) -> super::ParseResult<'_, Self> {
		let mut args = Vec::new();

		macro_rules! args {
			() => {
				while let Ok((new_input, arg)) = Arg::parse(input) {
					input = new_input;
					args.push(arg);
				}
			};
		}

		let (new_input, prenexes) = super::many0(Prenex::parse).parse(input)?;
		input = new_input;

		args!();

		let (new_input, cu) = nom::combinator::opt(Cu::parse)(input)?;
		input = new_input;

		// require selbri if cu is found
		let (new_input, selbri) = if cu.is_some() {
			nom::combinator::map(nom::combinator::cut(Selbri::parse), Some)(input)?
		} else {
			nom::combinator::opt(Selbri::parse)(input)?
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
			Self {
				prenexes,
				selbri,
				args: args.into_boxed_slice(),
				num_args_before_selbri,
			},
		))
	}
}

#[derive(Debug, Parse)]
pub struct Prenex {
	#[parse(with = "super::many0(Parse::parse)")]
	pub terms: Box<[Arg]>,
	pub zohu: Zohu,
}

#[derive(Debug, Parse)]
pub enum Arg {
	Tag(Tag),
	Sumti { fa: Option<Fa>, sumti: Sumti },
	Naku(Na, Ku),
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
	Na(Na),
	Tag(TagWord),
}

#[derive(Debug, Parse)]
#[repr(transparent)]
pub struct Selbri1(#[parse(with = "super::separated(true)")] Separated<Selbri2, Co>);

#[derive(Debug, Parse)]
#[repr(transparent)]
pub struct Selbri2(#[parse(with = "super::many1(Parse::parse)")] Box<[Selbri3]>);

pub type Selbri3 = Separated<Selbri4, JoikJek>;

pub type Selbri4 = Separated<Selbri5, (JoikJek, Bo)>;

pub type Selbri5 = Separated<Selbri6, Bo>;

#[derive(Debug, Parse)]
pub struct Selbri6 {
	#[parse(with = "super::many0(Parse::parse)")]
	pub connected: Box<[Selbri6ConnectedPre]>,
	pub last: TanruUnit,
}

#[derive(Debug, Parse)]
pub struct Selbri6ConnectedPre {
	pub nahe: Option<Nahe>,
	pub guha: Guha,
	#[cut]
	pub first: Selbri,
	pub gi: Gi,
}

#[derive(Debug, Parse)]
pub struct JoikJek {
	pub na: Option<Na>,
	pub se: Option<Se>,
	pub word: JoikJekWord,
	pub nai: Option<Nai>,
}

#[derive(Debug, Parse)]
pub enum JoikJekWord {
	Ja(Ja),
	Joi(Joi),
}

#[derive(Debug, Parse)]
#[repr(transparent)]
pub struct TanruUnit(#[parse(with = "super::separated(true)")] Separated<TanruUnit1, Cei>);

#[derive(Debug, Parse)]
pub struct TanruUnit1 {
	#[parse(with = "super::many0(Parse::parse)")]
	pub before: Box<[BeforeTanruUnit]>,
	pub inner: TanruUnit2,
	pub bound_arguments: Option<BoundArguments>,
}

#[derive(Debug, Parse)]
pub enum BeforeTanruUnit {
	Jai { jai: Jai, tag: Option<TagWord> },
	Nahe(Nahe),
	Se(Se),
}

#[derive(Debug, Parse)]
pub struct BoundArguments {
	pub be: Be,
	#[parse(with = "super::separated(true)")]
	pub args: Separated<Arg, Bei>,
	pub beho: Option<Beho>,
}

#[derive(Debug, Parse)]
pub enum TanruUnit2 {
	GroupedTanru {
		ke: Ke,
		#[cut]
		group: Selbri2, /* not `Selbri` because ke-ke'e groupings can't encompass co (CLL 5.8) nor tense, modal, and negation cmavo (CLL 5.13). `Selbri2` is inside co groupings (`Selbri1`) and na/tags (`Selbri`). */
		kehe: Option<Kehe>,
	},
	Gismu(Gismu),
	Lujvo(Lujvo),
	Fuhivla(Fuhivla),
	Goha {
		goha: Goha,
		raho: Option<Raho>,
	},
	Moi(
		#[parse(with = "super::many1(Parse::parse)")] Box<[NumberRest]>,
		Moi,
	),
	Me {
		me: Me,
		#[cut]
		inner: Sumti,
		mehu: Option<Mehu>,
	},
	Nu {
		#[parse(with = "super::separated(true)")]
		nus: Separated<(Nu, Option<Nai>), JoikJek>,
		#[cut]
		inner: Sentence,
		kei: Option<Kei>,
	},
	Nuha {
		nuha: Nuha,
		#[cut]
		operator: MeksoOperator,
	},
}

#[derive(Debug, Parse)]
pub struct Tag {
	pub words: TagWords,
	pub value: Option<TagValue>,
}

#[derive(Debug, Parse)]
pub enum TagValue {
	Ku(Ku),
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
	},
	TimeSpaceCaha {
		nahe: Option<Nahe>,
		#[parse(with = "super::many1(Parse::parse)")]
		inner: Box<[TimeSpaceCaha]>,
		ki: Option<Ki>,
	},
	Ki(Ki),
	Cuhe(Cuhe),
	Converted(Fiho, #[cut] Selbri, Option<Fehu>),
}

#[derive(Debug, Parse)]
pub enum TimeSpaceCaha {
	Time(Time),
	Space(Space),
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
pub struct SpaceIntervalProperty(Fehe, #[cut] IntervalProperty);

#[derive(Debug, Parse)]
pub struct SpaceMotion {
	pub mohi: Mohi,
	#[cut]
	pub offset: SpaceOffset,
}

#[derive(Debug, Parse)]
pub struct Sumti {
	pub inner: Sumti1,
	pub vuho_relative: Option<VuhoRelative>,
}

pub type Sumti1 = Separated<Sumti2, SumtiConnective>;
pub type Sumti2 = Separated<SumtiComponentOuter, (SumtiConnective, Bo)>;

#[derive(Debug, Parse)]
pub struct VuhoRelative {
	pub vuho: Vuho,
	pub relative_clauses: RelativeClauses,
}

#[derive(Debug, Parse)]
pub struct RelativeClauses(
	#[parse(with = "super::separated(true)")] Separated<RelativeClause, Zihe>,
);

#[derive(Debug, Parse)]
pub enum RelativeClause {
	Goi(GoiRelativeClause),
	Noi(NoiRelativeClause),
}

#[derive(Debug, Parse)]
pub struct GoiRelativeClause {
	pub goi: Goi,
	/// typical usage would match Arg::Sumti, but Arg::Tag is possible as well, such as in `la salis nesemau la betis cu se prami mi`
	pub inner: Arg,
	pub gehu: Option<Gehu>,
}

#[derive(Debug, Parse)]
pub struct NoiRelativeClause {
	pub noi: Noi,
	pub sentence: Sentence,
	pub kuho: Option<Kuho>,
}

#[derive(Debug, Parse)]
pub enum SumtiComponentOuter {
	Normal {
		quantifier: Option<Quantifier>,
		inner: SumtiComponent,
		relative_clauses: Option<RelativeClauses>,
	},
	SelbriShorthand {
		quantifier: Quantifier,
		inner: Selbri,
		ku: Option<Ku>,
		relative_clauses: Option<RelativeClauses>,
	},
}

#[derive(Debug, Parse)]
pub enum Quantifier {
	Mekso {
		vei: Vei,
		#[cut]
		mekso: Mekso,
		veho: Option<Veho>,
	},
	Number {
		number: Number,
		#[parse(not = "Moi")]
		boi: Option<Boi>,
	},
}

#[derive(Debug, Parse)]
pub struct Mekso;

#[derive(Debug, Parse)]
pub struct MeksoOperator;

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

pub type MiscNumbers = Box<[NumberRest]>;

#[derive(Debug, Parse)]
pub enum LerfuWord {
	By(By),
	Lau {
		lau: Lau,
		#[cut]
		by: By,
	},
	Tei {
		tei: Tei,
		#[cut]
		inner: Box<LerfuString>, // recursion avoided here
		#[cut]
		foi: Foi,
	},
}

#[derive(Debug, Parse)]
pub enum SumtiComponent {
	Koha(Koha),
	// it is important that this is checked before `la_sumti` because `la_sumti` `cut`s on `cmevla`
	Gadri(GadriSumti),
	La(LaSumti),
	Lohu(LohuSumti),
	Lu(LuSumti),
	Modified(ModifiedSumti),
	LerfuString(LerfuString, Option<Boi>),
	Zo(ZoSumti),
	Zoi(ZoiSumti),
	Li(Li, Mekso, Option<Loho>),
}

#[derive(Debug, Parse)]
pub struct LohuSumti {
	pub lohu: Lohu,
	#[parse(with = "super::many0(Token::parse)")]
	pub inner: Box<[Token]>,
	pub lehu: Lehu,
}

#[derive(Debug, Parse)]
pub struct LuSumti {
	pub lu: Lu,
	pub text: Text,
	pub lihu: Option<Lihu>,
}

#[derive(Debug, Parse)]
pub struct ModifiedSumti {
	pub modifier: SumtiModifier,
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
pub enum SumtiConnective {
	A(A),
	Joi(Joi),
}

#[derive(Debug, Parse)]
pub struct GadriSumti {
	pub gadri: Gadri,
	pub pe_shorthand: Option<Box<SumtiComponent>>, // recursion avoided here
	pub relative_clauses: Option<RelativeClauses>,
	pub inner: GadriSumtiInner,
	pub ku: Option<Ku>,
}

#[derive(Debug, Parse)]
pub enum Gadri {
	Le(Le),
	La(La),
}

#[derive(Debug, Parse)]
pub enum GadriSumtiInner {
	Selbri(Option<Quantifier>, Selbri, Option<RelativeClauses>),
	Sumti(Quantifier, #[cut] Sumti),
}

#[derive(Debug, Parse)]
pub struct LaSumti {
	pub la: La,
	#[cut]
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
