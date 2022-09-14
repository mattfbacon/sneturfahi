use super::{Bihi, Frees, Ga, Gaho, Gi, Giha, Guha, Ja, Joi, Na, Nai, Parse, Se, Tag, WithFree, A};

#[derive(Debug, Parse)]
pub struct NaSeTNai<T> {
	pub na: Option<Na>,
	pub se: Option<Se>,
	pub word: T,
	pub nai: Option<Nai>,
}

macro_rules! _ks {
	($($name:ident($inner:ident)),* $(,)?) => {
		$(
			#[derive(Debug, Parse)]
			#[repr(transparent)]
			pub struct $name(NaSeTNai<$inner>);
		)*
	}
}

_ks![Ek(A), Jek(Ja), Gihek(Giha)];

#[derive(Debug, Parse)]
pub struct Guhek {
	pub se: Option<Se>,
	pub word: Guha,
	pub nai: Option<Nai>,
	pub frees: Frees,
}

#[derive(Debug, Parse)]
pub struct Interval(pub Option<Se>, pub Bihi, pub Option<Nai>);

#[derive(Debug, Parse)]
pub enum Joik {
	SeJoiNai(Option<Se>, Joi, Option<Nai>),
	Interval(Interval),
	Gaho(Gaho, #[cut] Interval, #[cut] Gaho),
}

#[derive(Debug, Parse)]
pub struct Gik(Gi, Option<Nai>, Frees);

#[derive(Debug, Parse)]
pub enum Gek {
	SeGaNai(Option<Se>, Ga, Option<Nai>, Frees),
	JoikGi(Joik, Gi, Frees),
	TagGik(Tag, Gik),
}

#[derive(Debug, Parse)]
pub enum JoikJek {
	Joik(WithFree<Joik>),
	Jek(WithFree<Jek>),
}

#[derive(Debug, Parse)]
pub enum JoikEk {
	Joik(WithFree<Joik>),
	Ek(WithFree<Ek>),
}
