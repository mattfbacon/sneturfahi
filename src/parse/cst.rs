#![allow(missing_docs, missing_copy_implementations)]

use crate::lex::{Selmaho, Token};
use crate::span::Span;

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
	Cei,
	Cmevla,
	Co,
	Cu,
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
	Koha,
	Ku,
	Kuho,
	La,
	Lahe,
	Lau,
	Le,
	Lehu,
	Li,
	Lihu,
	Loho,
	Lohu,
	Lu,
	Luhu,
	Lujvo,
	Me,
	Mehu,
	Moi,
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
	Zoi,
}

pub struct Separated<Item, Separator> {
	pub first: Item,
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

#[derive(Debug)]
pub struct WithFree<Inner> {
	pub inner: Inner,
	pub free: Box<[Free]>,
}

// todo
#[derive(Debug)]
pub struct Free;

pub type Root = Text;

#[derive(Debug)]
pub struct Text {
	pub initial_i: Option<I>,
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

#[derive(Debug)]
pub struct Prenex {
	pub terms: Box<[Arg]>,
	pub zohu: Zohu,
}

#[derive(Debug)]
pub enum Arg {
	Tag(Tag),
	TagKu { tag: TagWord, ku: Ku },
	Sumti { fa: Option<Fa>, sumti: Sumti },
}

#[derive(Debug)]
pub struct Selbri {
	pub na: Box<[Na]>,
	pub components: Selbri1,
}

pub type Selbri1 = Separated<Selbri2, Co>;

pub type Selbri2 = Box<[Selbri3]>;

pub type Selbri3 = Separated<Selbri4, JoikJek>;

pub type Selbri4 = Separated<Selbri5, (JoikJek, Bo)>;

pub type Selbri5 = Separated<Selbri6, Bo>;

#[derive(Debug)]
pub enum Selbri6 {
	NotConnected(TanruUnit),
	Connected {
		nahe: Option<Nahe>,
		guha: Guha,
		first: Box<Selbri>, // recursion avoided here
		gi: Gi,
		second: Box<Self>,
	},
}

#[derive(Debug)]
pub struct JoikJek {
	pub na: Option<Na>,
	pub se: Option<Se>,
	pub word: JoikJekWord,
	pub nai: Option<Nai>,
}

#[derive(Debug)]
pub enum JoikJekWord {
	Ja(Ja),
	Joi(Joi),
}

pub type TanruUnit = Separated<TanruUnit1, Cei>;

#[derive(Debug)]
pub struct TanruUnit1 {
	pub before: Box<[BeforeTanruUnit]>,
	pub inner: TanruUnit2,
	pub bound_arguments: Option<BoundArguments>,
}

#[derive(Debug)]
pub enum BeforeTanruUnit {
	Jai { jai: Jai, tag: Option<TagWord> },
	Nahe(Nahe),
	Se(Se),
}

#[derive(Debug)]
pub struct BoundArguments {
	pub be: Be,
	pub args: Separated<Arg, Bei>,
	pub beho: Option<Beho>,
}

#[derive(Debug)]
pub enum TanruUnit2 {
	GroupedTanru {
		ke: Ke,
		group: Selbri2, /* not `Selbri` because ke-ke'e groupings can't encompass co (CLL 5.8). `Selbri2` is the rule immediately inside co groupings */
		kehe: Option<Kehe>,
	},
	Gismu(Gismu),
	Lujvo(Lujvo),
	Fuhivla(Fuhivla),
	Goha {
		goha: Goha,
		raho: Option<Raho>,
	},
	Moi(MiscNumbers, Moi),
	Me {
		me: Me,
		inner: Box<Sumti>, // large type avoided here
		mehu: Option<Mehu>,
	},
	Nu {
		nus: Separated<(Nu, Option<Nai>), JoikJek>,
		inner: Box<Sentence>, // large type avoided here
		kei: Option<Kei>,
	},
	Nuha {
		nuha: Nuha,
		operator: MeksoOperator,
	},
}

#[derive(Debug)]
pub struct Tag {
	pub words: TagWords,
	pub value: Option<Sumti>,
}

pub type TagWords = Separated<TagWord, JoikJek>;

#[derive(Debug)]
pub enum TagWord {
	Bai {
		se: Option<Se>,
		bai: Bai,
		nai: Option<Nai>,
	},
	Converted(Selbri),
}

#[derive(Debug)]
pub struct Sumti {
	pub inner: Separated<Separated<SumtiComponentOuter, (SumtiConnective, Bo)>, SumtiConnective>,
	pub vuho_relative: Option<VuhoRelative>,
}

#[derive(Debug)]
pub struct VuhoRelative {
	pub vuho: Vuho,
	pub relative_clauses: RelativeClauses,
}

pub type RelativeClauses = Separated<RelativeClause, Zihe>;

#[derive(Debug)]
pub enum RelativeClause {
	Goi(GoiRelativeClause),
	Noi(NoiRelativeClause),
}

#[derive(Debug)]
pub struct GoiRelativeClause {
	pub goi: Goi,
	/// typical usage would match Arg::Sumti, but Arg::Tag is possible as well, such as in `la salis nesemau la betis cu se prami mi`
	pub inner: Box<Arg>, // recursion avoided here
	pub gehu: Option<Gehu>,
}

#[derive(Debug)]
pub struct NoiRelativeClause {
	pub noi: Noi,
	pub sentence: Box<Sentence>, // large type avoided here
	pub kuho: Option<Kuho>,
}

#[derive(Debug)]
pub enum SumtiComponentOuter {
	Normal {
		quantifier: Option<Quantifier>,
		inner: SumtiComponent,
		relative_clauses: Option<RelativeClauses>,
	},
	SelbriShorthand {
		quantifier: Quantifier,
		inner: Box<Selbri>, // large type avoided here
		ku: Option<Ku>,
		relative_clauses: Option<RelativeClauses>,
	},
}

#[derive(Debug)]
pub enum Quantifier {
	Number {
		number: Number,
		boi: Option<Boi>,
	},
	Mekso {
		vei: Vei,
		mekso: Mekso,
		veho: Option<Veho>,
	},
}

// todo
#[derive(Debug)]
pub struct Mekso;

// todo
#[derive(Debug)]
pub struct MeksoOperator;

#[derive(Debug)]
pub struct Number {
	pub first: Pa,
	pub rest: Box<[NumberRest]>,
}

#[derive(Debug)]
pub enum NumberRest {
	Pa(Pa),
	Lerfu(LerfuWord),
}

#[derive(Debug)]
pub struct LerfuString {
	pub first: LerfuWord,
	pub rest: Box<[NumberRest]>,
}

pub type MiscNumbers = Box<[NumberRest]>;

#[derive(Debug)]
pub enum LerfuWord {
	By(By),
	Lau {
		lau: Lau,
		by: By,
	},
	Tei {
		tei: Tei,
		inner: Box<LerfuString>, // recursion avoided here
		foi: Foi,
	},
}

#[derive(Debug)]
pub enum SumtiComponent {
	Koha(Koha),
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

#[derive(Debug)]
pub struct LohuSumti {
	pub lohu: Lohu,
	pub inner: Box<[Token]>,
	pub lehu: Lehu,
}

#[derive(Debug)]
pub struct LuSumti {
	pub lu: Lu,
	pub text: Box<Text>,
	pub lihu: Option<Lihu>,
}

#[derive(Debug)]
pub struct ModifiedSumti {
	pub modifier: SumtiModifier,
	pub relative_clauses: Option<RelativeClauses>,
	pub sumti: Box<Sumti>,
	pub luhu: Option<Luhu>,
}

#[derive(Debug)]
pub enum SumtiModifier {
	Lahe(Lahe),
	NaheBo(Nahe, Bo),
}

#[derive(Debug)]
pub enum SumtiConnective {
	A(A),
	Joi(Joi),
}

#[derive(Debug)]
pub struct GadriSumti {
	pub gadri: Gadri,
	pub pe_shorthand: Option<Box<SumtiComponent>>, // recursion avoided here
	pub relative_clauses: Option<RelativeClauses>,
	pub inner: GadriSumtiInner,
	pub ku: Option<Ku>,
}

#[derive(Debug)]
pub enum Gadri {
	Le(Le),
	La(La),
}

#[derive(Debug)]
pub enum GadriSumtiInner {
	Selbri(Option<Quantifier>, Box<Selbri>, Option<RelativeClauses>),
	Sumti(Quantifier, Box<Sumti>),
}

#[derive(Debug)]
pub struct LaSumti {
	pub la: La,
	pub inner: Box<[Cmevla]>,
}

#[derive(Debug)]
pub struct ZoSumti {
	pub zo: Zo,
	pub quoted: Token,
}

#[derive(Debug)]
pub struct ZoiSumti {
	pub zoi: Zoi,
	pub starting_delimiter: Span,
	pub text: Span,
	pub ending_delimiter: Span,
}
