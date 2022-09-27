use macros::TreeNode;

use super::helpers::{many0, many1};
use super::{
	Bihe, Bo, Boi, Frees, Fuha, Gek, Gik, Johi, JoikEk, JoikJek, Ke, Kehe, Kuhe, Luhu, Maho,
	MiscNumbers, Mohe, Moi, Nahe, NaheGuhekTGik, Nahu, Nihe, Parse, Peho, Se, Selbri,
	SelbriLikeConnectedPost, Separated, Sumti, SumtiLikeConnectedPost, SumtiModifier, TagWords, Tehu,
	Veho, Vei, Vuhu, WithFree,
};

#[derive(Debug, Parse, TreeNode)]
pub enum Expression {
	ReversePolish(WithFree<Fuha>, ReversePolish),
	Normal(Separated<Separated<Expression1, (WithFree<Bihe>, Operator)>, Operator>),
}

// this representation is quite clunky and does not match the semantic hierarchy of the RP expression, but that's a problem for the AST.
#[derive(Debug, Parse, TreeNode)]
pub struct ReversePolish(pub Operand, #[parse(with = "many0")] pub Box<[RPTail]>);

#[derive(Debug, Parse, TreeNode)]
pub struct RPTail(pub ReversePolish, pub Operator);

#[derive(Debug, Parse, TreeNode)]
pub enum Expression1 {
	Operand(Operand),
	Forethought(ForethoughtExpression),
}

#[derive(Debug, Parse, TreeNode)]
pub struct ForethoughtExpression {
	pub peho: Option<WithFree<Peho>>,
	pub operator: Operator,
	#[parse(with = "many1")]
	pub operands: Box<[Expression1]>,
	pub kuhe: Option<Kuhe>,
	pub frees: Frees,
}

#[derive(Debug, Parse, TreeNode)]
pub struct Operand(
	pub Operand1,
	#[parse(with = "many0")] pub Box<[ConnectedOperand]>,
);

pub type ConnectedOperand = SumtiLikeConnectedPost<Operand1, Operand>;

#[derive(Debug, Parse, TreeNode)]
pub struct Operand1(pub Separated<Operand2, (JoikEk, Option<TagWords>, WithFree<Bo>)>);

#[derive(Debug, Parse, TreeNode)]
pub struct Operand2(
	#[parse(with = "many0")] pub Box<[Operand2ConnectedPre]>,
	pub Operand3,
);

#[derive(Debug, Parse, TreeNode)]
pub struct Operand2ConnectedPre(pub Gek, pub Operand, pub Gik);

#[derive(Debug, Parse, TreeNode)]
pub enum Operand3 {
	Nihe(WithFree<Nihe>, Selbri, Option<Tehu>, Frees),
	Mohe(WithFree<Mohe>, Sumti, Option<Tehu>, Frees),
	Johi(
		WithFree<Johi>,
		#[parse(with = "many1")] Box<[Expression1]>,
		Option<Tehu>,
		Frees,
	),
	Modified(OperandModifier, Operand, Option<Luhu>),
	Parenthesized(WithFree<Vei>, Expression, Option<Veho>, Frees),
	Number(MiscNumbers, #[parse(not = "Moi")] Option<Boi>, Frees),
}

pub type OperandModifier = SumtiModifier;

#[derive(Debug, Parse, TreeNode)]
pub struct Operator(
	pub Operator1,
	#[parse(with = "many0")] pub Box<[ConnectedOperator]>,
);

pub type ConnectedOperator = SelbriLikeConnectedPost<Operator1, Operator>;

#[derive(Debug, Parse, TreeNode)]
pub struct Operator1(
	#[parse(with = "many0")] pub Box<[NaheGuhekTGik<Self>]>,
	pub Operator2,
);

#[derive(Debug, Parse, TreeNode)]
pub struct Operator2(pub Separated<Operator3, (JoikJek, Option<TagWords>, WithFree<Bo>)>);

#[derive(Debug, Parse, TreeNode)]
pub enum Operator3 {
	Simple(OperatorComponent),
	Grouped(WithFree<Ke>, Operator, Kehe, Frees),
}

#[derive(Debug, Parse, TreeNode)]
pub struct OperatorComponent(
	#[parse(with = "many0")] Box<[OperatorComponentPre]>,
	OperatorComponent1,
);

#[derive(Debug, Parse, TreeNode)]
pub enum OperatorComponentPre {
	Nahe(WithFree<Nahe>),
	Se(WithFree<Se>),
}

#[derive(Debug, Parse, TreeNode)]
pub enum OperatorComponent1 {
	Maho(WithFree<Maho>, Expression, Option<Tehu>, Frees),
	Nahu(WithFree<Nahu>, Selbri, Option<Tehu>, Frees),
	Vuhu(WithFree<Vuhu>),
}
