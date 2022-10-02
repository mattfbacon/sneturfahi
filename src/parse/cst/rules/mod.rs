#![allow(missing_docs, missing_copy_implementations)]

// https://raw.githubusercontent.com/lojban/camxes-py/master/camxes_py/parsers/camxes_ilmen.peg

use bumpalo::Bump as Arena;
use macros::{Parse, TreeNode};

use crate::lex::{Selmaho, Token};
use crate::span::Span;

pub mod connectives;
mod helpers;
pub mod mekso;
pub mod selmaho;

use connectives::{Ek, Gek, Gihek, Gik, Guhek, Jek, Joik, JoikEk, JoikJek};
use helpers::{many0, many1, EitherOrBoth, Separated};
use mekso::{Expression as Mekso, Operator as MeksoOperator};
#[allow(clippy::wildcard_imports)]
use selmaho::*;

use super::parse_trait::{Parse, Result as ParseResult};
use crate::parse::tree_node::TreeNode;

#[derive(Debug, Parse, TreeNode)]
#[tree_node(passthrough_child)]
pub struct WithFree<'arena, Inner> {
	pub inner: Inner,
	pub frees: Frees<'arena>,
}

#[derive(Debug, Parse, TreeNode)]
#[tree_node(passthrough_child)]
pub struct Frees<'arena>(#[parse(with = "many0")] pub &'arena [Free<'arena>]);

pub type Root<'arena> = Text<'arena>;

#[derive(Debug, Parse, TreeNode)]
pub struct Text<'arena> {
	pub initial_indicators: Option<Indicators<'arena>>,
	pub initial_frees: Frees<'arena>,
	pub initial_paragraph_separator: Option<ParagraphSeparator<'arena>>,
	pub paragraphs: Option<Paragraphs<'arena>>,
	pub faho: Option<Faho<'arena>>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct Paragraphs<'arena>(Separated<'arena, Paragraph<'arena>, ParagraphSeparator<'arena>>);

#[derive(Debug, Parse, TreeNode)]
pub struct Paragraph<'arena> {
	pub initial_sentence_separator: Option<SentenceSeparator<'arena>>,
	pub sentences: ParagraphItems<'arena>,
}

pub type ParagraphItems<'arena> =
	Separated<'arena, ParagraphItem<'arena>, SentenceSeparator<'arena>>;

#[derive(Debug, Parse, TreeNode)]
pub enum ParagraphItem<'arena> {
	#[parse(must_consume)]
	Sentences(Sentences1<'arena>),
	Fragment(&'arena Fragment<'arena>),
	Empty(),
}

#[derive(Debug, Parse, TreeNode)]
pub enum Fragment<'arena> {
	// answer to ek connective question
	Ek(WithFree<'arena, Ek<'arena>>),
	// answer to jek connective question
	Jek(WithFree<'arena, Jek<'arena>>),
	// answer to ek or jek connective question
	Joik(WithFree<'arena, Joik<'arena>>),
	// answer to gihek connective question
	Gihek(WithFree<'arena, Gihek<'arena>>),
	// answer to number question
	// this is Quantifier rather than something that accepts MiscNumber because a lerfu string that starts with a letteral can be parsed as a sumti instead
	Number(Quantifier<'arena>),
	// answer to negation question?
	Na(Na<'arena>, #[parse(not = "Ja<'_>")] Frees<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct Sentences1<'arena>(
	#[parse(with = "many0")] pub &'arena [Prenex<'arena>],
	pub Separated<'arena, Sentences2<'arena>, ConnectedSentenceSeparator<'arena>>,
);
pub type Sentences2<'arena> = Separated<'arena, Sentences3<'arena>, CloseSentenceSeparator<'arena>>;

#[derive(Debug, Parse, TreeNode)]
pub enum Sentences3<'arena> {
	Grouped(
		Option<TagWords<'arena>>,
		WithFree<'arena, Tuhe<'arena>>,
		Paragraphs<'arena>,
		Option<Tuhu<'arena>>,
		Frees<'arena>,
	),
	Single(&'arena Sentence<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct ParagraphSeparator<'arena>(
	#[parse(with = "many1")] &'arena [Niho<'arena>],
	Frees<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceSeparator<'arena>(pub I<'arena>, #[parse(not = "Bu")] pub Frees<'arena>);

#[derive(Debug, Parse, TreeNode)]
pub struct ConnectedSentenceSeparator<'arena>(
	pub I<'arena>,
	pub JoikJek<'arena>,
	pub Frees<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct CloseSentenceSeparator<'arena>(
	pub I<'arena>,
	pub Option<JoikJek<'arena>>,
	pub Option<TagWords<'arena>>,
	pub Bo<'arena>,
	pub Frees<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct Sentence<'arena> {
	pub before_args: Args<'arena>,
	pub tail: Option<SentenceTail<'arena>>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail<'arena>(
	pub Option<Cu<'arena>>,
	pub Frees<'arena>,
	pub SentenceTail1<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail1<'arena>(
	pub SentenceTail2<'arena>,
	pub Option<SentenceTail1After<'arena>>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail1After<'arena>(
	Gihek<'arena>,
	Option<TagWords<'arena>>,
	Ke<'arena>,
	Frees<'arena>,
	&'arena SentenceTail1<'arena>,
	Option<Kehe<'arena>>,
	Frees<'arena>,
	TailArgs<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail2<'arena>(
	pub &'arena SentenceTail3<'arena>,
	pub Option<SentenceTail2After<'arena>>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail2Connective<'arena>(
	pub Gihek<'arena>,
	// `gi'e ke ...` must be parsed as SentenceTail1After, not a parenthesized tanru
	#[parse(not = "(Option<TagWords<'_>>, Ke<'_>)")] pub Frees<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail2After<'arena>(
	#[parse(with = "many1")] pub &'arena [(SentenceTail2Connective<'arena>, SentenceTail3<'arena>)],
	pub TailArgs<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail3<'arena>(
	pub &'arena SentenceTail4<'arena>,
	pub Option<SentenceTail3After<'arena>>,
);

pub type SentenceTail3Connective<'arena> = (
	Gihek<'arena>,
	Option<TagWords<'arena>>,
	Bo<'arena>,
	Frees<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct SentenceTail3After<'arena>(
	#[parse(with = "many1")] &'arena [(SentenceTail3Connective<'arena>, SentenceTail4<'arena>)],
	TailArgs<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub enum SentenceTail4<'arena> {
	Single(Selbri<'arena>, TailArgs<'arena>),
	Parenthesized(
		#[parse(with = "many0")] &'arena [WithFree<'arena, Na<'arena>>],
		Option<TagWords<'arena>>,
		WithFree<'arena, Ke<'arena>>,
		&'arena GekSentence<'arena>,
		Option<Kehe<'arena>>,
		Frees<'arena>,
	),
	Connected(&'arena GekSentence<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct GekSentence<'arena>(
	#[parse(with = "many0")] pub &'arena [WithFree<'arena, Na<'arena>>],
	pub Gek<'arena>,
	pub Subsentence<'arena>,
	pub Gik<'arena>,
	pub Subsentence<'arena>,
	pub TailArgs<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct Subsentence<'arena>(
	#[parse(with = "many0")] pub &'arena [Prenex<'arena>],
	Args<'arena>,
	SentenceTail<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
#[tree_node(passthrough_child)]
pub struct Args<'arena>(#[parse(with = "many0")] pub &'arena [Arg<'arena>]);

#[derive(Debug, Parse, TreeNode)]
#[tree_node(passthrough_child)]
pub struct TailArgs<'arena>(pub Args<'arena>, pub Option<Vau<'arena>>, pub Frees<'arena>);

#[derive(Debug, Parse, TreeNode)]
pub struct Prenex<'arena> {
	pub terms: Args<'arena>,
	pub zohu: Zohu<'arena>,
	pub frees: Frees<'arena>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct Arg<'arena>(
	Arg1<'arena>,
	#[parse(with = "many0")] &'arena [SumtiLikeConnectedPost<'arena, Arg1<'arena>, Self>],
);

pub type Arg1<'arena> = Separated<'arena, Arg2<'arena>, PeheConnective<'arena>>;

#[derive(Debug, Parse, TreeNode)]
pub struct PeheConnective<'arena>(WithFree<'arena, Pehe<'arena>>, JoikJek<'arena>);

pub type Arg2<'arena> = Separated<'arena, Arg3<'arena>, WithFree<'arena, Cehe<'arena>>>;

#[derive(Debug, Parse, TreeNode)]
pub enum Arg3<'arena> {
	Tag(Tag<'arena>),
	Sumti {
		fa: Option<WithFree<'arena, Fa<'arena>>>,
		sumti: Sumti<'arena>,
	},
	Naku(Na<'arena>, Ku<'arena>, Frees<'arena>),
	Termset(&'arena Termset<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
pub enum Termset<'arena> {
	Gek(Gek<'arena>, Args<'arena>, Gik<'arena>, Args<'arena>),
	NuhiGi(
		WithFree<'arena, Nuhi<'arena>>,
		Gek<'arena>,
		Args<'arena>,
		Option<Nuhu<'arena>>,
		Frees<'arena>,
		Gik<'arena>,
		Args<'arena>,
		Option<Nuhu<'arena>>,
		Frees<'arena>,
	),
	Nuhi(
		WithFree<'arena, Nuhi<'arena>>,
		Args<'arena>,
		Option<Nuhu<'arena>>,
		Frees<'arena>,
	),
}

#[derive(Debug, Parse, TreeNode)]
pub struct Selbri<'arena> {
	#[parse(with = "many0")]
	pub before: &'arena [SelbriBefore<'arena>],
	// all other `Separated` will use `false` for `should_cut`
	pub components: Selbri1<'arena>,
}

#[derive(Debug, Parse, TreeNode)]
pub enum SelbriBefore<'arena> {
	Na(WithFree<'arena, Na<'arena>>),
	Tag(TagWords<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
#[repr(transparent)]
pub struct Selbri1<'arena>(Separated<'arena, Selbri2<'arena>, WithFree<'arena, Co<'arena>>>);

#[derive(Debug, Parse, TreeNode)]
#[repr(transparent)]
pub struct Selbri2<'arena>(#[parse(with = "many1")] &'arena [Selbri3<'arena>]);

#[derive(Debug, Parse, TreeNode)]
pub struct Selbri3<'arena>(
	pub Selbri4<'arena>,
	#[parse(with = "many0")] pub &'arena [Selbri3ConnectedPost<'arena>],
);

pub type Selbri3ConnectedPost<'arena> =
	SelbriLikeConnectedPost<'arena, Selbri4<'arena>, Selbri2<'arena>>;

#[derive(Debug, Parse, TreeNode)]
pub enum SelbriLikeConnectedPost<'arena, Normal, Parenthesized> {
	Normal(JoikJek<'arena>, Normal),
	Parenthesized(
		JoikJek<'arena>,
		Option<TagWords<'arena>>,
		WithFree<'arena, Ke<'arena>>,
		Parenthesized,
		Option<Kehe<'arena>>,
		Frees<'arena>,
	),
}

#[derive(Debug, Parse, TreeNode)]
#[repr(transparent)]
pub struct Selbri4<'arena>(
	pub  Separated<
		'arena,
		Selbri5<'arena>,
		(
			JoikJek<'arena>,
			Option<TagWords<'arena>>,
			Bo<'arena>,
			Frees<'arena>,
		),
	>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct Selbri5<'arena>(
	#[parse(with = "many0")] pub &'arena [NaheGuhekTGik<'arena, Selbri<'arena>>],
	pub Selbri6<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct NaheGuhekTGik<'arena, T>(
	pub Option<Nahe<'arena>>,
	pub Frees<'arena>,
	pub Guhek<'arena>,
	pub T,
	pub Gik<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct Selbri6<'arena>(pub Separated<'arena, TanruUnit<'arena>, WithFree<'arena, Bo<'arena>>>);

#[derive(Debug, Parse, TreeNode)]
#[repr(transparent)]
pub struct TanruUnit<'arena>(
	pub Separated<'arena, TanruUnit1<'arena>, WithFree<'arena, Cei<'arena>>>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct TanruUnit1<'arena> {
	#[parse(with = "many0")]
	pub before: &'arena [BeforeTanruUnit<'arena>],
	pub inner: TanruUnit2<'arena>,
	pub bound_arguments: Option<BoundArguments<'arena>>,
}

#[derive(Debug, Parse, TreeNode)]
pub enum BeforeTanruUnit<'arena> {
	Jai {
		jai: WithFree<'arena, Jai<'arena>>,
		tag: Option<TagWords<'arena>>,
	},
	Nahe(WithFree<'arena, Nahe<'arena>>),
	Se(WithFree<'arena, Se<'arena>>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct BoundArguments<'arena> {
	pub be: WithFree<'arena, Be<'arena>>,
	pub args: Separated<'arena, Arg<'arena>, WithFree<'arena, Bei<'arena>>>,
	pub beho: Option<Beho<'arena>>,
	pub frees: Frees<'arena>,
}

#[derive(Debug, Parse, TreeNode)]
pub enum TanruUnit2<'arena> {
	GroupedTanru {
		ke: WithFree<'arena, Ke<'arena>>,
		group: Selbri2<'arena>, /* not `Selbri` because ke-ke'e groupings can't encompass co (CLL 5.8) nor tense, modal, and negation cmavo (CLL 5.13). `Selbri2` is inside co groupings (`Selbri1`) and na/tags (`Selbri`). */
		kehe: Option<Kehe<'arena>>,
		frees: Frees<'arena>,
	},
	Gismu(WithFree<'arena, Gismu<'arena>>),
	Lujvo(WithFree<'arena, Lujvo<'arena>>),
	Fuhivla(WithFree<'arena, Fuhivla<'arena>>),
	Goha {
		goha: Goha<'arena>,
		raho: Option<Raho<'arena>>,
		frees: Frees<'arena>,
	},
	Moi(MiscNumbers<'arena>, Moi<'arena>, Frees<'arena>),
	Me {
		me: WithFree<'arena, Me<'arena>>,
		inner: Sumti<'arena>,
		mehu: Option<Mehu<'arena>>,
		frees: Frees<'arena>,
		moi: Option<WithFree<'arena, Moi<'arena>>>,
	},
	Nu {
		nus: Separated<'arena, (Nu<'arena>, Option<Nai<'arena>>, Frees<'arena>), JoikJek<'arena>>,
		inner: &'arena Subsentence<'arena>,
		kei: Option<Kei<'arena>>,
		frees: Frees<'arena>,
	},
	Nuha {
		nuha: WithFree<'arena, Nuha<'arena>>,
		operator: MeksoOperator<'arena>,
	},
}

#[derive(Debug, Parse, TreeNode)]
pub struct Tag<'arena> {
	pub words: TagWords<'arena>,
	// e.g., "ri'agi broda gi brode" or "ri'agi ko'a gi ko'e". this rule would consume `ri'a` with no argument and leave just a `gi`.
	#[parse(not = "Gik<'_>", not = "Ke<'_>")]
	pub value: Option<TagValue<'arena>>,
}

#[derive(Debug, Parse, TreeNode)]
pub enum TagValue<'arena> {
	Ku(WithFree<'arena, Ku<'arena>>),
	Sumti(Sumti<'arena>),
}

pub type TagWords<'arena> = Separated<'arena, TagWord<'arena>, JoikJek<'arena>>;

#[derive(Debug, Parse, TreeNode)]
pub enum TagWord<'arena> {
	Bai {
		nahe: Option<Nahe<'arena>>,
		se: Option<Se<'arena>>,
		bai: Bai<'arena>,
		nai: Option<Nai<'arena>>,
		ki: Option<Ki<'arena>>,
		frees: Frees<'arena>,
	},
	TimeSpaceCaha {
		nahe: Option<Nahe<'arena>>,
		#[parse(with = "many1")]
		inner: &'arena [TimeSpaceCaha<'arena>],
		ki: Option<Ki<'arena>>,
		frees: Frees<'arena>,
	},
	Ki(Ki<'arena>, Frees<'arena>),
	Cuhe(Cuhe<'arena>, Frees<'arena>),
	Converted(
		WithFree<'arena, Fiho<'arena>>,
		Selbri<'arena>,
		Option<Fehu<'arena>>,
		Frees<'arena>,
	),
}

#[derive(Debug, Parse, TreeNode)]
pub enum TimeSpaceCaha<'arena> {
	Time(&'arena Time<'arena>),
	Space(&'arena Space<'arena>),
	Caha(Caha<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
#[parse(postcond("|time| time.zi.is_some() || !time.offset.is_empty() || time.duration.is_some() || !time.properties.is_empty()"))]
pub struct Time<'arena> {
	pub zi: Option<Zi<'arena>>,
	#[parse(with = "many0")]
	pub offset: &'arena [TimeOffset<'arena>],
	pub duration: Option<TimeDuration<'arena>>,
	#[parse(with = "many0")]
	pub properties: &'arena [TimeIntervalProperty<'arena>],
}

#[derive(Debug, Parse, TreeNode)]
pub struct TimeOffset<'arena> {
	pub pu: Pu<'arena>,
	pub nai: Option<Nai<'arena>>,
	pub zi: Option<Zi<'arena>>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct TimeDuration<'arena> {
	pub zeha: Zeha<'arena>,
	/// see CLL 10.5, specifically examples 10.26 through 10.29
	pub anchor: Option<(Pu<'arena>, Option<Nai<'arena>>)>,
}

#[derive(Debug, Parse, TreeNode)]
pub enum IntervalProperty<'arena> {
	Roi(Number<'arena>, Roi<'arena>, Option<Nai<'arena>>),
	Tahe(Tahe<'arena>, Option<Nai<'arena>>),
	Zaho(Zaho<'arena>, Option<Nai<'arena>>),
}

pub type TimeIntervalProperty<'arena> = IntervalProperty<'arena>;

#[derive(Debug, Parse, TreeNode)]
#[parse(postcond("|space| space.va.is_some() || !space.offset.is_empty() || space.interval.is_some() || space.motion.is_some()"))]
pub struct Space<'arena> {
	pub va: Option<Va<'arena>>,
	#[parse(with = "many0")]
	pub offset: &'arena [SpaceOffset<'arena>],
	pub interval: Option<SpaceInterval<'arena>>,
	pub motion: Option<SpaceMotion<'arena>>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct SpaceOffset<'arena>(Faha<'arena>, Option<Nai<'arena>>, Option<Va<'arena>>);

#[derive(Debug, Parse, TreeNode)]
pub enum SpaceInterval<'arena> {
	Interval {
		interval: EitherOrBoth<Veha<'arena>, Viha<'arena>>,
		direction: Option<(Faha<'arena>, Option<Nai<'arena>>)>,
		#[parse(with = "many0")]
		properties: &'arena [SpaceIntervalProperty<'arena>],
	},
	Properties(#[parse(with = "many1")] &'arena [SpaceIntervalProperty<'arena>]),
}

#[derive(Debug, Parse, TreeNode)]
pub struct SpaceIntervalProperty<'arena>(Fehe<'arena>, IntervalProperty<'arena>);

#[derive(Debug, Parse, TreeNode)]
pub struct SpaceMotion<'arena> {
	pub mohi: Mohi<'arena>,
	pub offset: SpaceOffset<'arena>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct Sumti<'arena> {
	pub inner: Sumti1<'arena>,
	pub vuho_relative: Option<VuhoRelative<'arena>>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct Sumti1<'arena>(
	pub Sumti2<'arena>,
	#[parse(with = "many0")]
	pub  &'arena [SumtiLikeConnectedPost<'arena, Sumti2<'arena>, Sumti<'arena>>],
);

#[derive(Debug, Parse, TreeNode)]
pub enum SumtiLikeConnectedPost<'arena, Normal, Parenthesized> {
	Normal(JoikEk<'arena>, Normal),
	Grouped(
		JoikEk<'arena>,
		Option<TagWords<'arena>>,
		WithFree<'arena, Ke<'arena>>,
		Parenthesized,
		Option<Kehe<'arena>>,
		Frees<'arena>,
	),
}

pub type Sumti2<'arena> = Separated<
	'arena,
	Sumti3<'arena>,
	(
		JoikEk<'arena>,
		Option<TagWords<'arena>>,
		Bo<'arena>,
		Frees<'arena>,
	),
>;

#[derive(Debug, Parse, TreeNode)]
pub struct VuhoRelative<'arena> {
	pub vuho: WithFree<'arena, Vuho<'arena>>,
	pub relative_clauses: RelativeClauses<'arena>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct RelativeClauses<'arena>(
	Separated<'arena, RelativeClause<'arena>, WithFree<'arena, Zihe<'arena>>>,
);

#[derive(Debug, Parse, TreeNode)]
pub enum RelativeClause<'arena> {
	Goi(GoiRelativeClause<'arena>),
	Noi(NoiRelativeClause<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct GoiRelativeClause<'arena> {
	pub goi: WithFree<'arena, Goi<'arena>>,
	/// typical usage would match Arg::Sumti, but Arg::Tag is possible as well, such as in `la salis nesemau la betis cu se prami mi`
	pub inner: Arg<'arena>,
	pub gehu: Option<Gehu<'arena>>,
	pub frees: Frees<'arena>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct NoiRelativeClause<'arena> {
	pub noi: WithFree<'arena, Noi<'arena>>,
	pub inner: &'arena Subsentence<'arena>,
	pub kuho: Option<Kuho<'arena>>,
	pub frees: Frees<'arena>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct Sumti3<'arena>(
	#[parse(with = "many0")] pub &'arena [Sumti3ConnectedPre<'arena>],
	Sumti4<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct Sumti3ConnectedPre<'arena>(pub Gek<'arena>, pub Sumti<'arena>, pub Gik<'arena>);

#[derive(Debug, Parse, TreeNode)]
pub enum Sumti4<'arena> {
	Normal {
		quantifier: Option<Quantifier<'arena>>,
		inner: &'arena SumtiComponent<'arena>,
		relative_clauses: Option<RelativeClauses<'arena>>,
	},
	SelbriShorthand {
		quantifier: Quantifier<'arena>,
		inner: Selbri<'arena>,
		ku: Option<Ku<'arena>>,
		frees: Frees<'arena>,
		relative_clauses: Option<RelativeClauses<'arena>>,
	},
}

#[derive(Debug, Parse, TreeNode)] // similar to part of `mekso::Operand3`
pub enum Quantifier<'arena> {
	Mekso(
		WithFree<'arena, Vei<'arena>>,
		Mekso<'arena>,
		Option<Veho<'arena>>,
		Frees<'arena>,
	),
	Number(
		Number<'arena>,
		#[parse(not = "Moi<'_>")] Option<Boi<'arena>>,
		Frees<'arena>,
	),
}

#[derive(Debug, Parse, TreeNode)]
pub struct Number<'arena> {
	pub first: Pa<'arena>,
	#[parse(with = "many0")]
	pub rest: &'arena [NumberRest<'arena>],
}

#[derive(Debug, Parse, TreeNode)]
pub enum NumberRest<'arena> {
	Pa(Pa<'arena>),
	Lerfu(LerfuWord<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct LerfuString<'arena> {
	pub first: LerfuWord<'arena>,
	#[parse(with = "many0")]
	pub rest: &'arena [NumberRest<'arena>],
}

#[derive(Debug, Parse, TreeNode)]
pub struct MiscNumbers<'arena>(#[parse(with = "many1")] &'arena [NumberRest<'arena>]);

#[derive(Debug, Parse, TreeNode)]
pub enum LerfuWord<'arena> {
	Lerfu(Lerfu<'arena>),
	Lau {
		lau: Lau<'arena>,
		lerfu: Lerfu<'arena>,
	},
	Tei {
		tei: Tei<'arena>,
		inner: &'arena LerfuString<'arena>, // recursion avoided here
		#[cut]
		foi: Foi<'arena>,
	},
}

#[derive(Debug, Parse, TreeNode)]
pub enum Lerfu<'arena> {
	Bu(
		Option<Bahe>,
		BuLerfu,
		#[parse(with = "many1")] &'arena [Bu],
		#[parse(with = "many0")] &'arena [Indicators<'arena>],
	),
	By(By<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
#[parse(
	postcond = "|Self(token)| !matches!(token.selmaho, Selmaho::Bu | Selmaho::Zei | Selmaho::Si | Selmaho::Su | Selmaho::Sa | Selmaho::Faho)"
)]
pub struct BuLerfu(Token);

pub type SumtiComponent<'arena> = WithFree<'arena, SumtiComponent1<'arena>>;

#[derive(Debug, Parse, TreeNode)]
pub enum SumtiComponent1<'arena> {
	Koha(Koha<'arena>),
	Gadri(&'arena GadriSumti<'arena>),
	La(LaSumti<'arena>),
	Lohu(LohuSumti<'arena>),
	Lu(LuSumti<'arena>),
	Modified(ModifiedSumti<'arena>),
	LerfuString(
		LerfuString<'arena>,
		#[parse(not = "Moi<'_>")] Option<Boi<'arena>>,
	),
	Zo(ZoSumti),
	Zoi(ZoiSumti),
	Li(
		WithFree<'arena, Li<'arena>>,
		Mekso<'arena>,
		Option<Loho<'arena>>,
	),
}

#[derive(Debug, TreeNode)]
pub struct LohuSumti<'arena> {
	pub lohu: Lohu,
	pub inner: &'arena [Token],
	pub lehu: Lehu<'arena>,
}

impl<'arena> Parse<'arena> for LohuSumti<'arena> {
	fn parse<'a: 'arena>(input: &'a [Token], arena: &'arena Arena) -> ParseResult<'a, Self> {
		nom::combinator::map(
			nom::sequence::tuple((
				|input| Parse::parse(input, arena),
				nom::combinator::cut(nom::multi::many_till(
					|input| Parse::parse(input, arena),
					|input| Parse::parse(input, arena),
				)),
			)),
			|(lohu, (inner, lehu))| Self {
				lohu,
				inner: arena.alloc_slice_fill_iter(inner.into_iter()),
				lehu,
			},
		)(input)
	}
}

#[derive(Debug, Parse, TreeNode)]
pub struct LuSumti<'arena> {
	pub lu: Lu,
	pub text: Text<'arena>,
	pub lihu: Option<Lihu<'arena>>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct ModifiedSumti<'arena> {
	pub modifier: WithFree<'arena, SumtiModifier<'arena>>,
	pub relative_clauses: Option<RelativeClauses<'arena>>,
	pub sumti: Sumti<'arena>,
	pub luhu: Option<Luhu<'arena>>,
}

#[derive(Debug, Parse, TreeNode)]
pub enum SumtiModifier<'arena> {
	Lahe(Lahe<'arena>),
	NaheBo(Nahe<'arena>, Bo<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct GadriSumti<'arena> {
	pub gadri: WithFree<'arena, Gadri<'arena>>,
	pub pre: GadriSumtiPre<'arena>,
	pub contents: GadriSumtiContents<'arena>,
	pub ku: Option<Ku<'arena>>,
}

#[derive(Debug, Parse, TreeNode)]
pub enum Gadri<'arena> {
	Le(Le<'arena>),
	La(La<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
#[tree_node(passthrough_child)]
pub enum GadriSumtiPre<'arena> {
	// order is important here
	Simple {
		pe_shorthand: Option<&'arena SumtiComponent<'arena>>, // recursion avoided here
		relative_clauses: Option<RelativeClauses<'arena>>,
	},
	Relative {
		relative_clauses: RelativeClauses<'arena>,
	},
	FullPre {
		pe_shorthand: &'arena Sumti<'arena>,
	},
}

#[derive(Debug, Parse, TreeNode)]
pub enum GadriSumtiContents<'arena> {
	Selbri(
		Option<Quantifier<'arena>>,
		Selbri<'arena>,
		Option<RelativeClauses<'arena>>,
	),
	Sumti(Quantifier<'arena>, Sumti<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct LaSumti<'arena> {
	pub la: La<'arena>,
	#[parse(with = "many1")]
	pub inner: &'arena [Cmevla<'arena>],
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
pub enum Free<'arena> {
	Sei(
		WithFree<'arena, Sei<'arena>>,
		Args<'arena>,
		Option<SeiTail<'arena>>,
		Option<WithFree<'arena, Sehu<'arena>>>,
	),
	Soi(
		WithFree<'arena, Soi<'arena>>,
		&'arena (Sumti<'arena>, Option<Sumti<'arena>>),
		Option<Sehu<'arena>>,
	),
	Vocative(Vocative<'arena>),
	Mai(MiscNumbers<'arena>, Mai<'arena>),
	To(To<'arena>, Text<'arena>, Option<Toi<'arena>>),
	Xi(Subscript<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct Vocative<'arena>(
	pub VocativeWords<'arena>,
	pub Option<RelativeClauses<'arena>>,
	pub VocativeValue<'arena>,
	pub Option<RelativeClauses<'arena>>,
	pub Option<Dohu<'arena>>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct SeiTail<'arena>(
	pub Option<Cu<'arena>>,
	pub Frees<'arena>,
	pub Selbri<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub enum VocativeWords<'arena> {
	Coi(
		#[parse(with = "many1")] &'arena [(Coi<'arena>, Option<Nai<'arena>>)],
		Option<Doi<'arena>>,
	),
	Doi(Doi<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
pub enum VocativeValue<'arena> {
	Selbri(&'arena Selbri<'arena>),
	Cmevla(#[parse(with = "many1")] &'arena [Cmevla<'arena>]),
	Sumti(Option<&'arena Sumti<'arena>>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct Subscript<'arena>(
	pub WithFree<'arena, Xi<'arena>>,
	pub WithFree<'arena, SubscriptValue<'arena>>,
);

#[derive(Debug, Parse, TreeNode)] // similar to part of `mekso::Operand3`
pub enum SubscriptValue<'arena> {
	Mekso(
		WithFree<'arena, Vei<'arena>>,
		Mekso<'arena>,
		Option<Veho<'arena>>,
		Frees<'arena>,
	),
	Number(
		MiscNumbers<'arena>,
		#[parse(not = "Moi<'_>")] Option<Boi<'arena>>,
		Frees<'arena>,
	),
}

#[derive(Debug, Parse, TreeNode)]
pub struct Indicators<'arena>(
	pub Option<Fuhe<'arena>>,
	#[parse(with = "many1")] pub &'arena [Indicator<'arena>],
);

#[derive(Debug, Parse, TreeNode)]
#[parse(not_after = "Bu")]
pub enum Indicator<'arena> {
	Ui(Ui<'arena>),
	Cai(Cai<'arena>),
	Nai(Nai<'arena>),
	Daho(Daho<'arena>),
	Fuho(Fuho<'arena>),
	Y(Y<'arena>),
}
