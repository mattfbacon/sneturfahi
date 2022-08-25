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
	Cmevla,
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
	Lau,
	Le,
	Lujvo,
	Me,
	Mehu,
	Na,
	Nahe,
	Nai,
	Noi,
	Nu,
	Pa,
	Se,
	Tei,
	Veho,
	Vei,
	Vuho,
	Zihe,
	Zo,
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
	pub selbri: Option<(Option<Cu>, Selbri)>,
	pub args: Box<[Arg]>,
	/// How many of `args` were before `selbri`.
	///
	/// Will be equal to `args.len()` if there is no selbri.
	pub num_args_before_selbri: usize,
}

#[derive(Debug)]
pub enum Arg {
	Tag(Tag),
	TagKu { tag: TagWord, ku: Ku },
	Sumti { fa: Option<Fa>, sumti: Sumti },
}

#[derive(Debug)]
pub struct Selbri {
	pub components: Box<[Separated<Separated<SelbriComponentOuter, (JoikJek, Bo)>, JoikJek>]>,
}

#[derive(Debug)]
pub enum SelbriComponentOuter {
	NotConnected(Separated<SelbriComponent, Bo>),
	Connected {
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

#[derive(Debug)]
pub struct SelbriComponent {
	pub before: Box<[BeforeSelbriComponent]>,
	pub word: SelbriWord,
	/// empty = no bound arguments
	pub bound_arguments: Option<BoundArguments>,
}

#[derive(Debug)]
pub enum BeforeSelbriComponent {
	Jai(Jai),
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
pub enum SelbriWord {
	GroupedTanru {
		ke: Ke,
		group: Box<Selbri>, // large type avoided here
		kehe: Option<Kehe>,
	},
	Gismu(Gismu),
	Lujvo(Lujvo),
	Fuhivla(Fuhivla),
	Me {
		me: Me,
		inner: Box<Sumti>, // large type avoided here
		mehu: Option<Mehu>,
	},
	Nu {
		nu: Nu,
		inner: Box<Sentence>, // large type avoided here
		kei: Option<Kei>,
	},
}

#[derive(Debug)]
pub struct Tag {
	pub words: Separated<TagWord, JoikJek>,
	pub value: Option<Sumti>,
}

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
	Le(LeSumti),
	La(LaSumti),
	Zo(ZoSumti),
	Zoi(ZoiSumti),
}

#[derive(Debug)]
pub enum SumtiConnective {
	A(A),
	Joi(Joi),
}

#[derive(Debug)]
pub struct LeSumti {
	pub le: Le,
	pub selbri: Selbri,
}

#[derive(Debug)]
pub struct LaSumti {
	pub la: La,
	pub inner: LaSumtiInner,
}

#[derive(Debug)]
pub enum LaSumtiInner {
	Cmevla(Box<[Cmevla]>),
	Selbri(Selbri),
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
