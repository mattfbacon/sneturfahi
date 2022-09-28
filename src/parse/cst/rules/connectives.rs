use macros::TreeNode;

use super::{Bihi, Frees, Ga, Gaho, Gi, Giha, Guha, Ja, Joi, Na, Nai, Parse, Se, TagWords, A};

#[derive(Debug, Parse, TreeNode)]
pub struct NaSeTNai<T> {
	pub na: Option<Na>,
	pub se: Option<Se>,
	pub word: T,
	pub nai: Option<Nai>,
}

macro_rules! _ks {
	($($name:ident($inner:ident)),* $(,)?) => {
		$(
			#[derive(Debug, Parse, TreeNode)]
			#[repr(transparent)]
			pub struct $name(NaSeTNai<$inner>);
		)*
	}
}

_ks![Ek(A), Jek(Ja), Gihek(Giha)];

#[derive(Debug, Parse, TreeNode)]
pub struct Guhek {
	pub se: Option<Se>,
	pub word: Guha,
	pub nai: Option<Nai>,
	pub frees: Frees,
}

#[derive(Debug, Parse, TreeNode)]
pub struct Interval(pub Option<Se>, pub Bihi, pub Option<Nai>);

#[derive(Debug, Parse, TreeNode)]
pub enum Joik {
	SeJoiNai(Option<Se>, Joi, Option<Nai>),
	Interval(Interval),
	Gaho(Gaho, Interval, #[cut] Gaho),
}

#[derive(Debug, Parse, TreeNode)]
pub struct Gik(Gi, Option<Nai>, Frees);

#[derive(Debug, Parse, TreeNode)]
pub enum Gek {
	SeGaNai(Option<Se>, Ga, Option<Nai>, Frees),
	JoikGi(Joik, Gi, Frees),
	TagGik(TagWords, Gik),
}

#[derive(Debug, Parse, TreeNode)]
pub enum JoikJek {
	Joik(Joik),
	Jek(Jek),
}

#[derive(Debug, Parse, TreeNode)]
pub enum JoikEk {
	Joik(Joik),
	Ek(Ek),
}
