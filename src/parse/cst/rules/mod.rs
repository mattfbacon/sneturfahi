#![allow(missing_docs, missing_copy_implementations)]

// https://raw.githubusercontent.com/lojban/camxes-py/master/camxes_py/parsers/camxes_ilmen.peg

use macros::{Parse, TreeNode};

use crate::lex::{Selmaho, Token};
use crate::span::Span;

pub mod connectives;
mod helpers;
pub mod mekso;
pub mod selmaho;

use connectives::{Gek, Gihek, Gik, Guhek, JoikEk, JoikJek};
use helpers::{many0, many1, EitherOrBoth, Separated};
use mekso::{Expression as Mekso, Operator as MeksoOperator};
#[allow(clippy::wildcard_imports)]
use selmaho::*;

use super::parse_trait::{Parse, Result as ParseResult};
use crate::parse::tree_node::TreeNode;

#[derive(Debug, Parse, TreeNode)]
#[tree_node(passthrough_child)]
pub struct WithFree<Inner> {
	pub inner: Inner,
	pub frees: Frees,
}

#[derive(Debug, Parse, TreeNode)]
#[tree_node(passthrough_child)]
pub struct Frees(#[parse(with = "many0")] pub Box<[Free]>);

pub type Root = Text;

#[derive(Debug, Parse, TreeNode)]
pub struct Text {
	pub initial_indicators: Option<Indicators>,
	pub initial_frees: Frees,
	pub initial_paragraph_separator: Option<ParagraphSeparator>,
	pub paragraphs: Option<Paragraphs>,
	pub faho: Option<Faho>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct Paragraphs(Separated<Paragraph, ParagraphSeparator>);

#[derive(Debug, Parse, TreeNode)]
pub struct Paragraph {
	pub initial_sentence_separator: Option<SentenceSeparator>,
	pub sentences: ParagraphItems,
}

pub type ParagraphItems = Separated<ParagraphItem, SentenceSeparator>;

#[derive(Debug, Parse, TreeNode)]
pub enum ParagraphItem {
	#[parse(must_consume)]
	Sentences(Sentences1),
	Fragment(Box<Fragment>),
	Empty(),
}

#[derive(Debug, Parse, TreeNode)]
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

#[derive(Debug, Parse, TreeNode)]
pub struct Sentences1(
	#[parse(with = "many0")] pub Box<[Prenex]>,
	pub Separated<Sentences2, ConnectedSentenceSeparator>,
);
pub type Sentences2 = Separated<Sentences3, CloseSentenceSeparator>;

#[derive(Debug, Parse, TreeNode)]
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

#[derive(Debug, Parse, TreeNode)]
pub struct ParagraphSeparator(#[parse(with = "many1")] Box<[Niho]>, Frees);

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceSeparator(pub I, #[parse(not = "Bu")] pub Frees);

#[derive(Debug, Parse, TreeNode)]
pub struct ConnectedSentenceSeparator(pub I, pub JoikJek, pub Frees);

#[derive(Debug, Parse, TreeNode)]
pub struct CloseSentenceSeparator(
	pub I,
	pub Option<JoikJek>,
	pub Option<TagWords>,
	pub Bo,
	pub Frees,
);

#[derive(Debug, Parse, TreeNode)]
pub struct Sentence {
	pub before_args: Args,
	pub tail: Option<SentenceTail>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail(pub Option<Cu>, pub Frees, pub SentenceTail1);

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail1(pub SentenceTail2, pub Option<SentenceTail1After>);

#[derive(Debug, Parse, TreeNode)]
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

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail2(pub Box<SentenceTail3>, pub Option<SentenceTail2After>);

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail2Connective(
	pub Gihek,
	// `gi'e ke ...` must be parsed as SentenceTail1After, not a parenthesized tanru
	#[parse(not = "(Option<TagWords>, Ke)")] pub Frees,
);

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail2After(
	#[parse(with = "many1")] pub Box<[(SentenceTail2Connective, SentenceTail3)]>,
	pub TailArgs,
);

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail3(pub Box<SentenceTail4>, pub Option<SentenceTail3After>);

pub type SentenceTail3Connective = (Gihek, Option<TagWords>, Bo, Frees);

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail3After(
	#[parse(with = "many1")] Box<[(SentenceTail3Connective, SentenceTail4)]>,
	TailArgs,
);

#[derive(Debug, Parse, TreeNode)]
pub enum SentenceTail4 {
	Single(Selbri, TailArgs),
	Parenthesized(
		#[parse(with = "many0")] Box<[WithFree<Na>]>,
		Option<TagWords>,
		WithFree<Ke>,
		Box<GekSentence>,
		Option<Kehe>,
		Frees,
	),
	Connected(Box<GekSentence>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct GekSentence(
	#[parse(with = "many0")] pub Box<[WithFree<Na>]>,
	pub Gek,
	pub Subsentence,
	pub Gik,
	pub Subsentence,
	pub TailArgs,
);

#[derive(Debug, Parse, TreeNode)]
pub struct Subsentence(
	#[parse(with = "many0")] pub Box<[Prenex]>,
	Args,
	SentenceTail,
);

#[derive(Debug, Parse, TreeNode)]
#[tree_node(passthrough_child)]
pub struct Args(#[parse(with = "many0")] pub Box<[Arg]>);

#[derive(Debug, Parse, TreeNode)]
#[tree_node(passthrough_child)]
pub struct TailArgs(pub Args, pub Option<Vau>, pub Frees);

#[derive(Debug, Parse, TreeNode)]
pub struct Prenex {
	pub terms: Args,
	pub zohu: Zohu,
	pub frees: Frees,
}

#[derive(Debug, Parse, TreeNode)]
pub struct Arg(
	Arg1,
	#[parse(with = "many0")] Box<[SumtiLikeConnectedPost<Arg1, Self>]>,
);

pub type Arg1 = Separated<Arg2, PeheConnective>;

#[derive(Debug, Parse, TreeNode)]
pub struct PeheConnective(WithFree<Pehe>, JoikJek);

pub type Arg2 = Separated<Arg3, WithFree<Cehe>>;

#[derive(Debug, Parse, TreeNode)]
pub enum Arg3 {
	Tag(Tag),
	Sumti {
		fa: Option<WithFree<Fa>>,
		sumti: Sumti,
	},
	Naku(Na, Ku, Frees),
	Termset(Box<Termset>),
}

#[derive(Debug, Parse, TreeNode)]
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

#[derive(Debug, Parse, TreeNode)]
pub struct Selbri {
	#[parse(with = "many0")]
	pub before: Box<[SelbriBefore]>,
	// all other `Separated` will use `false` for `should_cut`
	pub components: Selbri1,
}

#[derive(Debug, Parse, TreeNode)]
pub enum SelbriBefore {
	Na(WithFree<Na>),
	Tag(TagWords),
}

#[derive(Debug, Parse, TreeNode)]
#[repr(transparent)]
pub struct Selbri1(Separated<Selbri2, WithFree<Co>>);

#[derive(Debug, Parse, TreeNode)]
#[repr(transparent)]
pub struct Selbri2(#[parse(with = "many1")] Box<[Selbri3]>);

#[derive(Debug, Parse, TreeNode)]
pub struct Selbri3(
	pub Selbri4,
	#[parse(with = "many0")] pub Box<[Selbri3ConnectedPost]>,
);

pub type Selbri3ConnectedPost = SelbriLikeConnectedPost<Selbri4, Selbri2>;

#[derive(Debug, Parse, TreeNode)]
pub enum SelbriLikeConnectedPost<Normal, Parenthesized> {
	Normal(JoikJek, Normal),
	Parenthesized(
		JoikJek,
		Option<TagWords>,
		WithFree<Ke>,
		Parenthesized,
		Option<Kehe>,
		Frees,
	),
}

pub type Selbri4 = Separated<Selbri5, (JoikJek, Option<TagWords>, Bo, Frees)>;

#[derive(Debug, Parse, TreeNode)]
pub struct Selbri5(
	#[parse(with = "many0")] pub Box<[NaheGuhekTGik<Selbri>]>,
	pub Selbri6,
);

#[derive(Debug, Parse, TreeNode)]
pub struct NaheGuhekTGik<T>(pub Option<Nahe>, pub Frees, pub Guhek, pub T, pub Gik);

#[derive(Debug, Parse, TreeNode)]
pub struct Selbri6(pub Separated<TanruUnit, WithFree<Bo>>);

#[derive(Debug, Parse, TreeNode)]
#[repr(transparent)]
pub struct TanruUnit(pub Separated<TanruUnit1, WithFree<Cei>>);

#[derive(Debug, Parse, TreeNode)]
pub struct TanruUnit1 {
	#[parse(with = "many0")]
	pub before: Box<[BeforeTanruUnit]>,
	pub inner: TanruUnit2,
	pub bound_arguments: Option<BoundArguments>,
}

#[derive(Debug, Parse, TreeNode)]
pub enum BeforeTanruUnit {
	Jai {
		jai: WithFree<Jai>,
		tag: Option<TagWords>,
	},
	Nahe(WithFree<Nahe>),
	Se(WithFree<Se>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct BoundArguments {
	pub be: WithFree<Be>,
	pub args: Separated<Arg, WithFree<Bei>>,
	pub beho: Option<Beho>,
	pub frees: Frees,
}

#[derive(Debug, Parse, TreeNode)]
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

#[derive(Debug, Parse, TreeNode)]
pub struct Tag {
	pub words: TagWords,
	// e.g., "ri'agi broda gi brode" or "ri'agi ko'a gi ko'e". this rule would consume `ri'a` with no argument and leave just a `gi`.
	#[parse(not = "Gik", not = "Ke")]
	pub value: Option<TagValue>,
}

#[derive(Debug, Parse, TreeNode)]
pub enum TagValue {
	Ku(WithFree<Ku>),
	Sumti(Sumti),
}

pub type TagWords = Separated<TagWord, JoikJek>;

#[derive(Debug, Parse, TreeNode)]
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
		#[parse(with = "many1")]
		inner: Box<[TimeSpaceCaha]>,
		ki: Option<Ki>,
		frees: Frees,
	},
	Ki(Ki, Frees),
	Cuhe(Cuhe, Frees),
	Converted(WithFree<Fiho>, Selbri, Option<Fehu>, Frees),
}

#[derive(Debug, Parse, TreeNode)]
pub enum TimeSpaceCaha {
	Time(Time),
	Space(Box<Space>),
	Caha(Caha),
}

#[derive(Debug, Parse, TreeNode)]
#[parse(postcond("|time| time.zi.is_some() || !time.offset.is_empty() || time.duration.is_some() || !time.properties.is_empty()"))]
pub struct Time {
	pub zi: Option<Zi>,
	#[parse(with = "many0")]
	pub offset: Box<[TimeOffset]>,
	pub duration: Option<TimeDuration>,
	#[parse(with = "many0")]
	pub properties: Box<[TimeIntervalProperty]>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct TimeOffset {
	pub pu: Pu,
	pub nai: Option<Nai>,
	pub zi: Option<Zi>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct TimeDuration {
	pub zeha: Zeha,
	/// see CLL 10.5, specifically examples 10.26 through 10.29
	pub anchor: Option<(Pu, Option<Nai>)>,
}

#[derive(Debug, Parse, TreeNode)]
pub enum IntervalProperty {
	Roi(Number, Roi, Option<Nai>),
	Tahe(Tahe, Option<Nai>),
	Zaho(Zaho, Option<Nai>),
}

pub type TimeIntervalProperty = IntervalProperty;

#[derive(Debug, Parse, TreeNode)]
#[parse(postcond("|space| space.va.is_some() || !space.offset.is_empty() || space.interval.is_some() || space.motion.is_some()"))]
pub struct Space {
	pub va: Option<Va>,
	#[parse(with = "many0")]
	pub offset: Box<[SpaceOffset]>,
	pub interval: Option<SpaceInterval>,
	pub motion: Option<SpaceMotion>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct SpaceOffset(Faha, Option<Nai>, Option<Va>);

#[derive(Debug, Parse, TreeNode)]
pub enum SpaceInterval {
	Interval {
		interval: EitherOrBoth<Veha, Viha>,
		direction: Option<(Faha, Option<Nai>)>,
		#[parse(with = "many0")]
		properties: Box<[SpaceIntervalProperty]>,
	},
	Properties(#[parse(with = "many1")] Box<[SpaceIntervalProperty]>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct SpaceIntervalProperty(Fehe, IntervalProperty);

#[derive(Debug, Parse, TreeNode)]
pub struct SpaceMotion {
	pub mohi: Mohi,
	pub offset: SpaceOffset,
}

#[derive(Debug, Parse, TreeNode)]
pub struct Sumti {
	pub inner: Sumti1,
	pub vuho_relative: Option<VuhoRelative>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct Sumti1(
	pub Sumti2,
	#[parse(with = "many0")] pub Box<[SumtiLikeConnectedPost<Sumti2, Sumti>]>,
);

#[derive(Debug, Parse, TreeNode)]
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

#[derive(Debug, Parse, TreeNode)]
pub struct VuhoRelative {
	pub vuho: WithFree<Vuho>,
	pub relative_clauses: RelativeClauses,
}

#[derive(Debug, Parse, TreeNode)]
pub struct RelativeClauses(Separated<RelativeClause, WithFree<Zihe>>);

#[derive(Debug, Parse, TreeNode)]
pub enum RelativeClause {
	Goi(GoiRelativeClause),
	Noi(NoiRelativeClause),
}

#[derive(Debug, Parse, TreeNode)]
pub struct GoiRelativeClause {
	pub goi: WithFree<Goi>,
	/// typical usage would match Arg::Sumti, but Arg::Tag is possible as well, such as in `la salis nesemau la betis cu se prami mi`
	pub inner: Arg,
	pub gehu: Option<Gehu>,
	pub frees: Frees,
}

#[derive(Debug, Parse, TreeNode)]
pub struct NoiRelativeClause {
	pub noi: WithFree<Noi>,
	pub inner: Box<Subsentence>,
	pub kuho: Option<Kuho>,
	pub frees: Frees,
}

#[derive(Debug, Parse, TreeNode)]
pub struct Sumti3(
	#[parse(with = "many0")] pub Box<[Sumti3ConnectedPre]>,
	Sumti4,
);

#[derive(Debug, Parse, TreeNode)]
pub struct Sumti3ConnectedPre(pub Gek, pub Sumti, pub Gik);

#[derive(Debug, Parse, TreeNode)]
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

#[derive(Debug, Parse, TreeNode)] // similar to part of `mekso::Operand3`
pub enum Quantifier {
	Mekso(WithFree<Vei>, Mekso, Option<Veho>, Frees),
	Number(Number, #[parse(not = "Moi")] Option<Boi>, Frees),
}

#[derive(Debug, Parse, TreeNode)]
pub struct Number {
	pub first: Pa,
	#[parse(with = "many0")]
	pub rest: Box<[NumberRest]>,
}

#[derive(Debug, Parse, TreeNode)]
pub enum NumberRest {
	Pa(Pa),
	Lerfu(LerfuWord),
}

#[derive(Debug, Parse, TreeNode)]
pub struct LerfuString {
	pub first: LerfuWord,
	#[parse(with = "many0")]
	pub rest: Box<[NumberRest]>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct MiscNumbers(#[parse(with = "many1")] Box<[NumberRest]>);

#[derive(Debug, Parse, TreeNode)]
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

#[derive(Debug, Parse, TreeNode)]
pub enum Lerfu {
	Bu(
		Option<Bahe>,
		BuLerfu,
		#[parse(with = "many1")] Box<[Bu]>,
		#[parse(with = "many0")] Box<[Indicators]>,
	),
	By(By),
}

#[derive(Debug, Parse, TreeNode)]
#[parse(
	postcond = "|Self(token)| !matches!(token.selmaho, Selmaho::Bu | Selmaho::Zei | Selmaho::Si | Selmaho::Su | Selmaho::Sa | Selmaho::Faho)"
)]
pub struct BuLerfu(Token);

pub type SumtiComponent = WithFree<SumtiComponent1>;

#[derive(Debug, Parse, TreeNode)]
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

#[derive(Debug, TreeNode)]
pub struct LohuSumti {
	pub lohu: Lohu,
	pub inner: Box<[Token]>,
	pub lehu: Lehu,
}

impl Parse for LohuSumti {
	fn parse(input: &[Token]) -> ParseResult<'_, Self> {
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

#[derive(Debug, Parse, TreeNode)]
pub struct LuSumti {
	pub lu: Lu,
	pub text: Text,
	pub lihu: Option<Lihu>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct ModifiedSumti {
	pub modifier: WithFree<SumtiModifier>,
	pub relative_clauses: Option<RelativeClauses>,
	pub sumti: Sumti,
	pub luhu: Option<Luhu>,
}

#[derive(Debug, Parse, TreeNode)]
pub enum SumtiModifier {
	Lahe(Lahe),
	NaheBo(Nahe, Bo),
}

#[derive(Debug, Parse, TreeNode)]
pub struct GadriSumti {
	pub gadri: WithFree<Gadri>,
	pub pre: GadriSumtiPre,
	pub contents: GadriSumtiContents,
	pub ku: Option<Ku>,
}

#[derive(Debug, Parse, TreeNode)]
pub enum Gadri {
	Le(Le),
	La(La),
}

#[derive(Debug, Parse, TreeNode)]
#[tree_node(passthrough_child)]
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

#[derive(Debug, Parse, TreeNode)]
pub enum GadriSumtiContents {
	Selbri(Option<Quantifier>, Selbri, Option<RelativeClauses>),
	Sumti(Quantifier, Sumti),
}

#[derive(Debug, Parse, TreeNode)]
pub struct LaSumti {
	pub la: La,
	#[parse(with = "many1")]
	pub inner: Box<[Cmevla]>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct ZoSumti {
	pub zo: Zo,
	pub quoted: Token,
}

#[derive(Debug, Parse, TreeNode)]
pub struct ZoiSumti {
	pub zoi: Zoi,
	pub starting_delimiter: ZoiDelimiter,
	pub text: ZoiText,
	pub ending_delimiter: ZoiDelimiter,
}

#[derive(Debug, Parse)]
#[repr(transparent)]
pub struct ZoiDelimiter(Span);

impl TreeNode for ZoiDelimiter {
	fn name(&self) -> &'static str {
		"ZoiDelimiter"
	}

	fn experimental(&self) -> bool {
		false
	}

	fn start_location(&self) -> Option<crate::span::Location> {
		Some(self.0.start)
	}

	fn end_location(&self) -> Option<crate::span::Location> {
		Some(self.0.end)
	}

	fn for_each_child<'a>(&'a self, _: &mut dyn FnMut(&'a dyn TreeNode)) {}
}

#[derive(Debug, Parse)]
#[repr(transparent)]
pub struct ZoiText(Span);

impl TreeNode for ZoiText {
	fn name(&self) -> &'static str {
		"ZoiText"
	}

	fn experimental(&self) -> bool {
		false
	}

	fn start_location(&self) -> Option<crate::span::Location> {
		Some(self.0.start)
	}

	fn end_location(&self) -> Option<crate::span::Location> {
		Some(self.0.end)
	}

	fn for_each_child<'a>(&'a self, _: &mut dyn FnMut(&'a dyn TreeNode)) {}
}

#[derive(Debug, Parse, TreeNode)]
pub enum Free {
	Sei(WithFree<Sei>, Args, Option<SeiTail>, Option<WithFree<Sehu>>),
	Soi(WithFree<Soi>, Box<(Sumti, Option<Sumti>)>, Option<Sehu>),
	Vocative(Vocative),
	Mai(MiscNumbers, Mai),
	To(To, Text, Option<Toi>),
	Xi(Subscript),
}

#[derive(Debug, Parse, TreeNode)]
pub struct Vocative(
	pub VocativeWords,
	pub Option<RelativeClauses>,
	pub VocativeValue,
	pub Option<RelativeClauses>,
	pub Option<Dohu>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct SeiTail(pub Option<Cu>, pub Frees, pub Selbri);

#[derive(Debug, Parse, TreeNode)]
pub enum VocativeWords {
	Coi(
		#[parse(with = "many1")] Box<[(Coi, Option<Nai>)]>,
		Option<Doi>,
	),
	Doi(Doi),
}

#[derive(Debug, Parse, TreeNode)]
pub enum VocativeValue {
	Selbri(Box<Selbri>),
	Cmevla(#[parse(with = "many1")] Box<[Cmevla]>),
	Sumti(Option<Box<Sumti>>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct Subscript(pub WithFree<Xi>, pub WithFree<SubscriptValue>);

#[derive(Debug, Parse, TreeNode)] // similar to part of `mekso::Operand3`
pub enum SubscriptValue {
	Mekso(WithFree<Vei>, Mekso, Option<Veho>, Frees),
	Number(MiscNumbers, #[parse(not = "Moi")] Option<Boi>, Frees),
}

#[derive(Debug, Parse, TreeNode)]
pub struct Indicators(
	pub Option<Fuhe>,
	#[parse(with = "many1")] pub Box<[Indicator]>,
);

#[derive(Debug, Parse, TreeNode)]
#[parse(not_after = "Bu")]
pub enum Indicator {
	Ui(Ui),
	Cai(Cai),
	Nai(Nai),
	Daho(Daho),
	Fuho(Fuho),
	Y(Y),
}
