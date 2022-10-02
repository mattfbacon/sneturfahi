use macros::TreeNode;

use super::{Bihi, Frees, Ga, Gaho, Gi, Giha, Guha, Ja, Joi, Na, Nai, Parse, Se, TagWords, A};

#[derive(Debug, Parse, TreeNode)]
pub struct NaSeTNai<'arena, T>(
	pub Option<Na<'arena>>,
	pub Option<Se<'arena>>,
	pub T,
	pub Option<Nai<'arena>>,
);

macro_rules! _ks {
	($($name:ident($inner:ident)),* $(,)?) => {
		$(
			#[derive(Debug, Parse, TreeNode)]
			#[repr(transparent)]
			pub struct $name<'arena>(NaSeTNai<'arena, $inner<'arena>>);
		)*
	}
}

_ks![Ek(A), Jek(Ja), Gihek(Giha)];

#[derive(Debug, Parse, TreeNode)]
pub struct Guhek<'arena>(
	pub Option<Se<'arena>>,
	pub Guha<'arena>,
	pub Option<Nai<'arena>>,
	pub Frees<'arena>,
);

#[derive(Debug, Parse, TreeNode)]
pub struct Interval<'arena>(
	pub Option<Se<'arena>>,
	pub Bihi<'arena>,
	pub Option<Nai<'arena>>,
);

#[derive(Debug, Parse, TreeNode)]
pub enum Joik<'arena> {
	SeJoiNai(Option<Se<'arena>>, Joi<'arena>, Option<Nai<'arena>>),
	Interval(Interval<'arena>),
	Gaho(Gaho<'arena>, Interval<'arena>, #[cut] Gaho<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
pub struct Gik<'arena>(Gi<'arena>, Option<Nai<'arena>>, Frees<'arena>);

#[derive(Debug, Parse, TreeNode)]
pub enum Gek<'arena> {
	SeGaNai(
		Option<Se<'arena>>,
		Ga<'arena>,
		Option<Nai<'arena>>,
		Frees<'arena>,
	),
	JoikGi(Joik<'arena>, Gi<'arena>, Frees<'arena>),
	TagGik(TagWords<'arena>, Gik<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
pub enum JoikJek<'arena> {
	Joik(Joik<'arena>),
	Jek(Jek<'arena>),
}

#[derive(Debug, Parse, TreeNode)]
pub enum JoikEk<'arena> {
	Joik(Joik<'arena>, Frees<'arena>),
	Ek(Ek<'arena>, Frees<'arena>),
}
