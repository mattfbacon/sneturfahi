use macros::TreeNode;

use super::helpers::many0;
use super::Indicators;
use crate::lex::{Selmaho, Token};
use crate::parse::cst::error::{Error, WithLocation};
use crate::parse::cst::parse_trait::{Parse, Result as ParseResult};
use crate::parse::tree_node::TreeNode;
use crate::span::{Location, Span};

pub(in crate::parse::cst::rules) trait SelmahoTypeRaw:
	TryFrom<Token, Error = Error> + TryFrom<Option<Token>, Error = Error>
{
}

pub(in crate::parse::cst::rules) fn selmaho_raw<T: SelmahoTypeRaw>(
	input: &[Token],
) -> ParseResult<'_, T> {
	let mut input = input.iter();
	T::try_from(input.next().copied())
		.map(|matched| (input.as_slice(), matched))
		.map_err(|error| {
			nom::Err::Error(WithLocation {
				location: input.as_slice(),
				error,
			})
		})
}

macro_rules! token_types {
	(@inner $name:ident) => {paste::paste!{
		#[derive(Debug)]
		pub struct [<$name Inner>] {
			pub experimental: bool,
			pub span: Span,
		}

		impl TreeNode for [<$name Inner>] {
			fn name(&self) -> &'static str {
				Selmaho::$name.as_repr()
			}

			fn experimental(&self) -> bool {
				self.experimental
			}

			fn start_location(&self) -> Option<Location> {
				Some(self.span.start)
			}

			fn end_location(&self) -> Option<Location> {
				Some(self.span.end)
			}

			fn for_each_child<'a>(&'a self, _: &mut dyn FnMut(&'a dyn TreeNode)) {}
		}
	}};
	(@raw $name:ident) => {
		#[derive(Debug)]
		pub struct $name {
			pub experimental: bool,
			pub span: Span,
		}

		impl TryFrom<Token> for $name {
			type Error = Error;

			fn try_from(token: Token) -> Result<Self, Error> {
				if token.selmaho == Selmaho::$name {
					Ok(Self {
						experimental: token.experimental,
						span: token.span,
					})
				} else {
					Err(Error::ExpectedGot {
						expected: (&[Selmaho::$name] as &[Selmaho]).into(),
						got: Some(token),
					})
				}
			}
		}

		impl TryFrom<Option<Token>> for $name {
			type Error = Error;

			fn try_from(token: Option<Token>) -> Result<Self, Error> {
				token
					.ok_or(Error::ExpectedGot {
						expected: (&[Selmaho::$name] as &[Selmaho]).into(),
						got: None,
					})
					.and_then(Self::try_from)
			}
		}

		impl SelmahoTypeRaw for $name {}

		impl Parse for $name {
			fn parse<'a>(input: &'a [Token]) -> ParseResult<'a, Self> {
				selmaho_raw::<$name>(input)
			}
		}

		impl TreeNode for $name {
			fn name(&self) -> &'static str {
				Selmaho::$name.as_repr()
			}

			fn experimental(&self) -> bool {
				self.experimental
			}

			fn start_location(&self) -> Option<Location> {
				Some(self.span.start)
			}

			fn end_location(&self) -> Option<Location> {
				Some(self.span.end)
			}

			fn for_each_child<'a>(&'a self, _: &mut dyn FnMut(&'a dyn TreeNode)) {}
		}
	};
	(@no_indicators $name:ident) => {paste::paste!{
		token_types!(@inner $name);

		#[derive(Debug, TreeNode)]
		pub struct $name {
			pub bahe: Box<[Bahe]>,
			pub inner: [<$name Inner>],
		}

		impl TryFrom<Token> for $name {
			type Error = Error;

			fn try_from(token: Token) -> Result<Self, Error> {
				if token.selmaho == Selmaho::$name {
					Ok(Self {
						bahe: Box::new([]),
						inner: [<$name Inner>] {
							experimental: token.experimental,
							span: token.span,
						},
					})
				} else {
					Err(Error::ExpectedGot {
						expected: (&[Selmaho::$name] as &[Selmaho]).into(),
						got: Some(token),
					})
				}
			}
		}

		impl TryFrom<Option<Token>> for $name {
			type Error = Error;

			fn try_from(token: Option<Token>) -> Result<Self, Error> {
				token.ok_or(Error::ExpectedGot { expected: (&[Selmaho::$name] as &[Selmaho]).into(), got: None }).and_then(Self::try_from)
			}
		}
		impl SelmahoTypeRaw for $name {}

		impl Parse for $name {
			fn parse<'a>(input: &'a [Token]) -> ParseResult<'a, Self> {
				let (input, bahe) = nom::Parser::parse(&mut many0(Bahe::parse), input)?;
				let (input, mut matched) = selmaho_raw::<Self>(input)?;
				matched.bahe = bahe;
				Ok((input, matched))
			}
		}
	}};
	(@single $name:ident) => {paste::paste!{
		token_types!(@inner $name);

		#[derive(Debug, TreeNode)]
		pub struct $name {
			pub bahe: Box<[Bahe]>,
			pub inner: [<$name Inner>],
			pub indicators: Option<Box<Indicators>>,
		}

		impl TryFrom<Token> for $name {
			type Error = Error;

			fn try_from(token: Token) -> Result<Self, Error> {
				if token.selmaho == Selmaho::$name {
					Ok(Self {
						bahe: Box::new([]),
						inner: [<$name Inner>] {
							experimental: token.experimental,
							span: token.span,
						},
						indicators: None,
					})
				} else {
					Err(Error::ExpectedGot {
						expected: (&[Selmaho::$name] as &[Selmaho]).into(),
						got: Some(token),
					})
				}
			}
		}

		impl TryFrom<Option<Token>> for $name {
			type Error = Error;

			fn try_from(token: Option<Token>) -> Result<Self, Error> {
				token.ok_or(Error::ExpectedGot { expected: (&[Selmaho::$name] as &[Selmaho]).into(), got: None }).and_then(Self::try_from)
			}
		}
		impl SelmahoTypeRaw for $name {}

		impl Parse for $name {
			fn parse<'a>(input: &'a [Token]) -> ParseResult<'a, Self> {
				let (input, bahe) = nom::Parser::parse(&mut many0(Bahe::parse), input)?;
				let (input, mut matched) = selmaho_raw::<Self>(input)?;
				let (input, indicators) = <Option<Box<Indicators>>>::parse(input)?;
				matched.bahe = bahe;
				matched.indicators = indicators;
				Ok((input, matched))
			}
		}
	}};
	($(,)?) => {};
	($(,)? $name:ident $($rest:tt)*) => {
		token_types!(@single $name);
		token_types!($($rest)*);
	};
	($(,)? #[no_indicators] $name:ident $($rest:tt)*) => {
		token_types!(@no_indicators $name);
		token_types!($($rest)*);
	};
	($(,)? #[raw] $name:ident $($rest:tt)*) => {
		token_types!(@raw $name);
		token_types!($($rest)*);
	};
}

token_types! {
	A,
	#[raw] Bahe,
	Bai,
	Be,
	Beho,
	Bei,
	Bihe,
	Bihi,
	Bo,
	Boi,
	#[raw] Bu,
	By,
	Caha,
	Cai,
	Cehe,
	Cei,
	Cmevla,
	Co,
	Coi,
	Giha,
	Cu,
	Cuhe,
	Daho,
	Dohu,
	Doi,
	Tuhe,
	Tuhu,
	Fa,
	Faha,
	Faho,
	Fehe,
	Fehu,
	Fiho,
	Foi,
	Fuha,
	#[no_indicators] Fuhe,
	Fuhivla,
	Fuho,
	Ga,
	Gaho,
	Gehu,
	Gi,
	Gismu,
	Goha,
	Goi,
	Niho,
	Guha,
	I,
	Ja,
	Jai,
	Johi,
	Joi,
	Ke,
	Kehe,
	Kei,
	Ki,
	Koha,
	Ku,
	Kuhe,
	Kuho,
	La,
	Lahe,
	Lau,
	Le,
	Lehu,
	Li,
	Lihu,
	Loho,
	#[raw] Lohu,
	#[raw] Lu,
	Luhu,
	Lujvo,
	Maho,
	Mai,
	Me,
	Mehu,
	Mohe,
	Mohi,
	Moi,
	Na,
	Nahe,
	Nahu,
	Nai,
	Nihe,
	Noi,
	Nu,
	Nuha,
	Nuhi,
	Nuhu,
	Pa,
	Pehe,
	Peho,
	Pu,
	Raho,
	Roi,
	Se,
	Sehu,
	Sei,
	Soi,
	Tahe,
	Tehu,
	Tei,
	To,
	Toi,
	Ui,
	Va,
	Vau,
	Veha,
	Veho,
	Vei,
	Viha,
	Vuho,
	Vuhu,
	Xi,
	Y,
	Zaho,
	Zeha,
	Zi,
	Zihe,
	#[raw] Zo,
	Zohu,
	#[raw] Zoi,
}
