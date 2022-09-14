use super::{
	Bihe, Bo, Boi, Frees, Fuha, Gek, Gik, Johi, JoikEk, JoikJek, Ke, Kehe, Kuhe, Luhu, Maho, Mekso,
	MiscNumbers, Mohe, Moi, Nahe, NaheGuhekTGik, Nahu, Nihe, Parse, Peho, Se, Selbri, Separated,
	Sumti, SumtiLikeConnectedPost, SumtiModifier, TagWords, Tehu, Veho, Vei, Vuhu, WithFree,
};

#[derive(Debug, Parse)]
pub enum Expression {
	ReversePolish(WithFree<Fuha>, ReversePolish),
	Normal(Separated<Separated<Expression1, (WithFree<Bihe>, Operator)>, Operator>),
}

#[derive(Debug, Parse)]
pub struct ReversePolish;

#[derive(Debug, Parse)]
pub enum Expression1 {
	Operand(Operand),
	Forethought(ForethoughtExpression),
}

#[derive(Debug, Parse)]
pub struct ForethoughtExpression {
	pub peho: Option<WithFree<Peho>>,
	pub operator: Operator,
	#[parse(with = "super::super::many1(Parse::parse)")]
	pub operands: Box<[Expression1]>,
	pub kuhe: Option<Kuhe>,
	pub frees: Frees,
}

#[derive(Debug, Parse)]
pub struct Operand(
	pub Operand1,
	#[parse(with = "super::super::many0(Parse::parse)")] pub Box<[ConnectedOperand]>,
);

pub type ConnectedOperand = SumtiLikeConnectedPost<Operand>;

#[derive(Debug, Parse)]
pub struct Operand1(pub Separated<Operand2, (JoikEk, Option<TagWords>, WithFree<Bo>)>);

#[derive(Debug, Parse)]
pub struct Operand2(
	#[parse(with = "super::super::many0(Parse::parse)")] pub Box<[Operand2ConnectedPre]>,
	pub Operand3,
);

#[derive(Debug, Parse)]
pub struct Operand2ConnectedPre(pub Gek, pub Operand, pub Gik);

#[derive(Debug, Parse)]
pub enum Operand3 {
	Nihe(WithFree<Nihe>, #[cut] Selbri, Option<Tehu>, Frees),
	Mohe(WithFree<Mohe>, #[cut] Sumti, Option<Tehu>, Frees),
	Johi(
		WithFree<Johi>,
		#[cut]
		#[parse(with = "super::super::many1(Parse::parse)")]
		Box<[Expression1]>,
		Option<Tehu>,
		Frees,
	),
	Modified(OperandModifier, Operand, Option<Luhu>),
	Parenthesized(WithFree<Vei>, #[cut] Mekso, Option<Veho>, Frees),
	Number(MiscNumbers, #[parse(not = "Moi")] Option<Boi>, Frees),
}

pub type OperandModifier = SumtiModifier;

#[derive(Debug, Parse)]
pub struct Operator(
	pub Operator1,
	#[parse(with = "super::super::many0(Parse::parse)")] pub Box<[ConnectedOperator]>,
);

pub type ConnectedOperator = SumtiLikeConnectedPost<Operator>;

#[derive(Debug, Parse)]
pub struct Operator1(
	#[parse(with = "super::super::many0(Parse::parse)")] pub Box<[NaheGuhekTGik<Self>]>,
	pub Operator2,
);

#[derive(Debug, Parse)]
pub struct Operator2(pub Separated<Operator3, (JoikJek, Option<TagWords>, WithFree<Bo>)>);

#[derive(Debug, Parse)]
pub enum Operator3 {
	Simple(OperatorComponent),
	Grouped(WithFree<Ke>, #[cut] Operator, Kehe, Frees),
}

#[derive(Debug, Parse)]
pub struct OperatorComponent(
	#[parse(with = "super::super::many0(Parse::parse)")] Box<[OperatorComponentPre]>,
	OperatorComponent1,
);

#[derive(Debug, Parse)]
pub enum OperatorComponentPre {
	Nahe(WithFree<Nahe>),
	Se(WithFree<Se>),
}

#[derive(Debug, Parse)]
pub enum OperatorComponent1 {
	Maho(WithFree<Maho>, #[cut] Mekso, Option<Tehu>, Frees),
	Nahu(WithFree<Nahu>, #[cut] Selbri, Option<Tehu>, Frees),
	Vuhu(WithFree<Vuhu>),
}
