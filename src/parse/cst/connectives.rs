use super::{Bihi, Ga, Gaho, Gi, Giha, Guha, Ja, Joi, Na, Nai, Parse, Se, Tag, A};

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

_ks![Ek(A), Guhek(Guha), Jek(Ja), Gihek(Giha)];

#[derive(Debug, Parse)]
pub struct Interval(pub Option<Se>, pub Bihi, pub Option<Nai>);

#[derive(Debug, Parse)]
pub enum Joik {
	SeJoiNai(Option<Se>, Joi, Option<Nai>),
	Interval(Interval),
	Gaho(Gaho, #[cut] Interval, #[cut] Gaho),
}

#[derive(Debug, Parse)]
pub struct Gik(Gi, Option<Nai>);

#[derive(Debug, Parse)]
pub enum Gek {
	SeGaNai(Option<Se>, Ga, Option<Nai>),
	JoikGi(Joik, Gi),
	TagGik(Tag, Gik),
}

#[derive(Debug, Parse)]
pub enum JoikJek {
	Joik(Joik),
	Jek(Jek),
}

#[derive(Debug, Parse)]
pub enum JoikEk {
	Joik(Joik),
	Ek(Ek),
}
