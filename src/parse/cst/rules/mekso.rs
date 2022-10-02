use macros::TreeNode;

use super::helpers::{many0, many1};
use super::{
	Bihe, Bo, Boi, Frees, Fuha, Gek, Gik, Johi, JoikEk, JoikJek, Ke, Kehe, Kuhe, Luhu, Maho,
	MiscNumbers, Mohe, Moi, Nahe, NaheGuhekTGik, Nahu, Nihe, Parse, Peho, Se, Selbri,
	SelbriLikeConnectedPost, Separated, Sumti, SumtiLikeConnectedPost, SumtiModifier, TagWords, Tehu,
	Veho, Vei, Vuhu, WithFree,
};

#[derive(Debug, Parse, TreeNode)]
pub enum Expression<'arena> {
	ReversePolish(WithFree<'arena, Fuha<'arena>>, ReversePolish<'arena>),
	Normal(
		Separated<
			'arena,
			Separated<'arena, Expression1<'arena>, (WithFree<'arena, Bihe<'arena>>, Operator<'arena>)>,
			Operator<'arena>,
		>,
	),
}

// this representation is quite clunky and does not match the semantic hierarchy of the RP expression, but that's a problem for the AST.
#[derive(Debug, Parse, TreeNode)]
pub struct ReversePolish<'arena>(
	pub Operand<'arena>,
	#[parse(with = "many0")] pub &'arena [RPTail<'arena>],
);

#[derive(Debug, Parse, TreeNode)]
pub struct RPTail<'arena>(pub ReversePolish<'arena>, pub Operator<'arena>);

#[derive(Debug, Parse, TreeNode)]
pub enum Expression1<'arena> {
	Operand(Operand<'arena>),
	Forethought(ForethoughtExpression<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct ForethoughtExpression<'arena> {
	pub peho: Option<WithFree<'arena, Peho<'arena>>>,
	pub operator: Operator<'arena>,
	#[parse(with = "many1")]
	pub operands: &'arena [Expression1<'arena>],
	pub kuhe: Option<Kuhe<'arena>>,
	pub frees: Frees<'arena>,
}

#[derive(Debug, Parse, TreeNode)]
pub struct Operand<'arena>(
	pub Operand1<'arena>,
	#[parse(with = "many0")] pub &'arena [ConnectedOperand<'arena>],
);

pub type ConnectedOperand<'arena> =
	SumtiLikeConnectedPost<'arena, Operand1<'arena>, Operand<'arena>>;

#[derive(Debug, Parse, TreeNode)]
pub struct Operand1<'arena>(
	pub  Separated<
		'arena,
		Operand2<'arena>,
		(
			JoikEk<'arena>,
			Option<TagWords<'arena>>,
			WithFree<'arena, Bo<'arena>>,
		),
	>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct Operand2<'arena>(
	#[parse(with = "many0")] pub &'arena [Operand2ConnectedPre<'arena>],
	pub Operand3<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct Operand2ConnectedPre<'arena>(pub Gek<'arena>, pub Operand<'arena>, pub Gik<'arena>);

#[derive(Debug, Parse, TreeNode)]
pub enum Operand3<'arena> {
	Nihe(
		WithFree<'arena, Nihe<'arena>>,
		Selbri<'arena>,
		Option<Tehu<'arena>>,
		Frees<'arena>,
	),
	Mohe(
		WithFree<'arena, Mohe<'arena>>,
		Sumti<'arena>,
		Option<Tehu<'arena>>,
		Frees<'arena>,
	),
	Johi(
		WithFree<'arena, Johi<'arena>>,
		#[parse(with = "many1")] &'arena [Expression1<'arena>],
		Option<Tehu<'arena>>,
		Frees<'arena>,
	),
	Modified(
		OperandModifier<'arena>,
		Operand<'arena>,
		Option<Luhu<'arena>>,
	),
	Parenthesized(
		WithFree<'arena, Vei<'arena>>,
		Expression<'arena>,
		Option<Veho<'arena>>,
		Frees<'arena>,
	),
	Number(
		MiscNumbers<'arena>,
		#[parse(not = "Moi<'_>")] Option<Boi<'arena>>,
		Frees<'arena>,
	),
}

pub type OperandModifier<'arena> = SumtiModifier<'arena>;

#[derive(Debug, Parse, TreeNode)]
pub struct Operator<'arena>(
	pub Operator1<'arena>,
	#[parse(with = "many0")] pub &'arena [ConnectedOperator<'arena>],
);

pub type ConnectedOperator<'arena> =
	SelbriLikeConnectedPost<'arena, Operator1<'arena>, Operator<'arena>>;

#[derive(Debug, Parse, TreeNode)]
pub struct Operator1<'arena>(
	#[parse(with = "many0")] pub &'arena [NaheGuhekTGik<'arena, Self>],
	pub Operator2<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct Operator2<'arena>(
	pub  Separated<
		'arena,
		Operator3<'arena>,
		(
			JoikJek<'arena>,
			Option<TagWords<'arena>>,
			WithFree<'arena, Bo<'arena>>,
		),
	>,
);

#[derive(Debug, Parse, TreeNode)]
pub enum Operator3<'arena> {
	Simple(OperatorComponent<'arena>),
	Grouped(
		WithFree<'arena, Ke<'arena>>,
		Operator<'arena>,
		Kehe<'arena>,
		Frees<'arena>,
	),
}

#[derive(Debug, Parse, TreeNode)]
pub struct OperatorComponent<'arena>(
	#[parse(with = "many0")] &'arena [OperatorComponentPre<'arena>],
	OperatorComponent1<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub enum OperatorComponentPre<'arena> {
	Nahe(WithFree<'arena, Nahe<'arena>>),
	Se(WithFree<'arena, Se<'arena>>),
}

#[derive(Debug, Parse, TreeNode)]
pub enum OperatorComponent1<'arena> {
	Maho(
		WithFree<'arena, Maho<'arena>>,
		Expression<'arena>,
		Option<Tehu<'arena>>,
		Frees<'arena>,
	),
	Nahu(
		WithFree<'arena, Nahu<'arena>>,
		Selbri<'arena>,
		Option<Tehu<'arena>>,
		Frees<'arena>,
	),
	Vuhu(WithFree<'arena, Vuhu<'arena>>),
}
