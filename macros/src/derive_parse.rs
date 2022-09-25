use proc_macro2::{Span, TokenStream, TokenTree};
use proc_macro_error::abort;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::token::Comma;
use syn::{
	parse_macro_input, parse_quote, AttrStyle, Attribute, Data, DeriveInput, Field, Fields,
	GenericParam, Generics, Ident, Index, Lit, LitStr, Meta, MetaList, MetaNameValue, NestedMeta,
};

mod paths {
	use proc_macro2::TokenStream;
	use quote::quote;

	pub(super) fn trait_() -> TokenStream {
		quote!(crate::parse::cst::parse_trait::Parse)
	}

	pub(super) fn result() -> TokenStream {
		quote!(crate::parse::cst::parse_trait::Result)
	}

	pub(super) fn error() -> TokenStream {
		quote!(crate::parse::cst::error::Error)
	}

	pub(super) fn token() -> TokenStream {
		quote!(crate::lex::Token)
	}
}

pub fn derive_parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;
	add_trait_bounds(&mut input.generics);
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let body = implement_parse(&input.data, &input.attrs, &name);

	let trait_path = paths::trait_();
	let result_path = paths::result();
	let token_path = paths::token();
	let assert_parse = quote_spanned! {Span::mixed_site()=>
		fn assert_parse<T: #trait_path>() -> impl FnMut(&[#token_path]) -> #result_path<'_, T> {
			T::parse
		}
	};

	quote! {
		#[automatically_derived]
		#[allow(unused_qualifications)]
		impl #impl_generics #trait_path for #name #ty_generics #where_clause {
			fn parse(input: &[#token_path]) -> #result_path<'_, Self> {
				#assert_parse

				#body
			}
		}
	}
	.into()
}

fn add_trait_bounds(generics: &mut Generics) {
	let trait_path = paths::trait_();
	for param in &mut generics.params {
		if let GenericParam::Type(type_param) = param {
			type_param.bounds.push(parse_quote!(#trait_path));
		}
	}
}

fn implement_parse(data: &Data, attrs: &[Attribute], ident: &Ident) -> TokenStream {
	let attrs = ContainerAttributes::get(attrs);

	let trait_path = paths::trait_();
	let inner = if let Some(with) = attrs.with {
		quote_spanned!(with.span()=> nom::Parser::parse(&mut (#with)(#trait_path::parse), input))
	} else {
		match data {
			Data::Struct(data) => implement_struct(&data.fields, ident, attrs.must_consume, quote!(Self)),
			Data::Enum(data) => {
				if data.variants.is_empty() {
					abort!(
						data.brace_token.span,
						"at least one variant is required. maybe you wanted a unit struct?"
					);
				}

				let branches = data.variants.iter().map(|variant| {
					let attrs = VariantAttributes::get(&variant.attrs);
					if matches!(variant.fields, Fields::Unit) {
						abort!(variant.span(), "enum unit variants are not allowed");
					}
					let variant_ident = &variant.ident;
					let body = implement_struct(
						&variant.fields,
						ident,
						attrs.must_consume,
						quote_spanned!(variant_ident.span()=> Self::#variant_ident),
					);
					quote_spanned! {variant.span()=>
						|input| #body
					}
				});

				if attrs.longest {
					let branches = branches.map(|branch| {
						quote_spanned! {branch.span()=>
							match (#branch)(input) {
								ret @ Err(nom::Err::Failure(..)) => return ret,
								Err(nom::Err::Error(error)) => Err(error),
								Err(nom::Err::Incomplete(..)) => unreachable!("no streaming parsers used"),
								Ok(parsed) => Ok(parsed),
							}
						}
					});
					quote! {
						let results = [
							#(#branches,)*
						];
						(if results.iter().all(|result| result.is_err()) {
							Err(nom::Err::Error(results.into_iter().map(Result::unwrap_err).reduce(nom::error::ParseError::or).unwrap())) // we know there is at least one variant
						} else {
							Ok(results.into_iter().filter_map(Result::ok).min_by_key(|(rest, _parsed)| nom::InputLength::input_len(rest)).unwrap()) // ditto
						})
					}
				} else {
					let inner = quote! {
						nom::branch::alt((
							#(#branches,)*
						))(input)
					};

					if attrs.must_consume {
						wrap_must_consume(&ident.to_string(), inner)
					} else {
						inner
					}
				}
			}
			Data::Union(union) => abort!(
				union.union_token.span(),
				"`Parse` can only be derived on structs and enums"
			),
		}
	};

	let inner = if attrs.post_conds.is_empty() {
		inner
	} else {
		let check_post_conds = attrs.post_conds.iter().map(|post_cond| {
			let PostCond {
				cond_original,
				cond,
				reason,
			} = post_cond;
			let reason = reason
				.as_ref()
				.cloned()
				.unwrap_or_else(|| LitStr::new(&cond_original.value(), Span::call_site()));
			let error = paths::error();
			quote_spanned! {cond_original.span()=>
				if !((#cond) as fn(&Self) -> bool)(&value) {
					return Err(
						nom::Err::Error(
							#error::PostConditionFailed(#reason).with_location(input)
						)
					);
				}
			}
		});

		quote!({
			let (rest, value): (_, Self) = #inner?;

			#(#check_post_conds)*

			Ok((rest, value))
		})
	};

	if attrs.after_nots.is_empty() {
		inner
	} else {
		let after_nots = attrs.after_nots;
		let after_nots = after_nots
			.iter()
			.map(|not| quote_spanned! {not.span() => nom::combinator::not(assert_parse::<#not>())});
		quote! {
			nom::combinator::map(
				nom::sequence::tuple((
					|input| { #inner },
					#(#after_nots,)*
				)),
				|(inner, ..)| inner
			)(input)
		}
	}
}

fn implement_struct(
	fields: &Fields,
	ident: &Ident,
	must_consume: bool,
	constructor: TokenStream,
) -> TokenStream {
	let (elements, field_names, named) = match fields {
		Fields::Named(fields) => {
			let elements = make_elements(&fields.named);
			let field_names: Vec<_> = fields
				.named
				.iter()
				.map(|field| field.ident.as_ref().unwrap().to_token_stream())
				.collect();
			(elements, field_names, true)
		}
		Fields::Unnamed(fields) => {
			let elements = make_elements(&fields.unnamed);
			let field_names: Vec<_> = fields
				.unnamed
				.iter()
				.enumerate()
				.map(|(idx, field)| format_ident!("field_{idx}", span = field.span()).into_token_stream())
				.collect();
			(elements, field_names, false)
		}
		Fields::Unit => {
			let todo_string = format!("unimplemented parsing rule {}", ident);
			return quote! {
				todo!(#todo_string)
			};
		}
	};

	if field_names.is_empty() {
		if must_consume {
			abort!(fields.span(), "an empty struct/variant will never consume");
		}

		quote_spanned!(constructor.span()=> Ok((input, #constructor{})))
	} else {
		let fields = if named {
			quote!({ #(#field_names,)* })
		} else {
			quote!((#(#field_names,)*))
		};

		let inner = quote! {
			nom::combinator::map(nom::sequence::tuple((
				#(#elements,)*
			)), |(#(#field_names,)*)| #constructor #fields)(input)
		};

		if must_consume {
			wrap_must_consume(&constructor.to_token_stream().to_string(), inner)
		} else {
			inner
		}
	}
}

fn wrap_must_consume(name: &str, inner: TokenStream) -> TokenStream {
	let span = inner.span();
	let error_path = paths::error();
	let mut inner = proc_macro2::Group::new(proc_macro2::Delimiter::None, inner);
	inner.set_span(span);
	quote_spanned! {span=> {
		let result = #inner;
		if result.as_ref().map_or(false, |(rest, parsed)| nom::InputLength::input_len(rest) == nom::InputLength::input_len(&input)) {
			Err(nom::Err::Error(#error_path::Empty(#name).with_location(input)))
		} else {
			result
		}
	}}
}

fn make_elements(i: &Punctuated<Field, Comma>) -> impl Iterator<Item = TokenStream> + '_ {
	let trait_path = paths::trait_();

	i.iter().map(move |field| {
		let attrs = FieldAttributes::get(&field.attrs);
		let ty = &field.ty;
		let ty_span = ty.span();

		let inner = attrs.with.map_or_else(
			|| quote_spanned!(ty_span=> assert_parse::<#ty>()),
			|with| quote_spanned!(with.span()=> (#with)(#trait_path::parse)),
		);

		let actual = if attrs.cut {
			quote_spanned! {ty_span=> nom::combinator::cut(#inner) }
		} else {
			inner
		};

		let nots = attrs.nots;
		let after_nots = attrs.after_nots;
		if !nots.is_empty() || !after_nots.is_empty() {
			let idx = Index::from(nots.len());
			let nots = nots
				.iter()
				.map(|not| quote_spanned!(not.span() => nom::combinator::not(assert_parse::<#not>())));
			let after_nots = after_nots
				.iter()
				.map(|not| quote_spanned!(not.span() => nom::combinator:not(assert_parse::<#not>())));
			quote! {
				nom::combinator::map(
					nom::sequence::tuple((
						#(#nots,)*
						#actual,
						#(#after_nots,)*
					)),
					|tuple| tuple.#idx
				)
			}
		} else {
			actual
		}
	})
}

#[derive(Default)]
struct ContainerAttributes {
	after_nots: Vec<TokenStream>,
	with: Option<TokenStream>,
	post_conds: Vec<PostCond>,
	longest: bool,
	must_consume: bool,
}

impl ContainerAttributes {
	fn get(attrs: &[Attribute]) -> Self {
		let mut ret = Self::default();

		for (span, attr) in get_parse_attributes(attrs) {
			match attr {
				ParseAttribute::Not(..) => abort!(span, "`not` attribute can only be used on fields. maybe you meant to put the `not` on the first field?"),
				ParseAttribute::NotAfter(not) => ret.after_nots.push(not),
				ParseAttribute::Cut => abort!(span, "`cut` attribute can only be used on fields. maybe you meant to put the `cut` on the first field?"),
				ParseAttribute::PostCond(post_cond) => ret.post_conds.push(post_cond),
				ParseAttribute::With(new_with) => if ret.with.replace(new_with).is_some() {
					abort!(span, "multiple `with` attributes are not allowed");
				}
				ParseAttribute::Longest => ret.longest = true,
				ParseAttribute::NonEmpty => ret.must_consume = true,
			}
		}

		ret
	}
}

#[derive(Default)]
struct FieldAttributes {
	cut: bool,
	nots: Vec<TokenStream>,
	after_nots: Vec<TokenStream>,
	with: Option<TokenStream>,
}

impl FieldAttributes {
	fn get(attrs: &[Attribute]) -> Self {
		let mut ret = Self::default();

		for (span, attr) in get_parse_attributes(attrs) {
			match attr {
				ParseAttribute::Not(not) => ret.nots.push(not),
				ParseAttribute::NotAfter(not) => ret.after_nots.push(not),
				ParseAttribute::Cut => ret.cut = true,
				ParseAttribute::PostCond(_) => {
					abort!(span, "`postcond` attribute is only allowed on containers")
				}
				ParseAttribute::With(with) => {
					if ret.with.replace(with).is_some() {
						abort!(span, "multiple `with` attributes are not allowed")
					}
				}
				ParseAttribute::Longest => {
					abort!(span, "`longest` attribute is only allowed on containers")
				}
				ParseAttribute::NonEmpty => {
					abort!(
						span,
						"`must_consume` attribute is only allowed on enum variants"
					)
				}
			}
		}

		ret
	}
}

struct PostCond {
	cond_original: LitStr,
	cond: TokenStream,
	reason: Option<LitStr>,
}

struct VariantAttributes {
	must_consume: bool,
}

impl VariantAttributes {
	fn get(attrs: &[Attribute]) -> Self {
		let mut must_consume = false;

		for (span, attr) in get_parse_attributes(attrs) {
			match attr {
				ParseAttribute::NonEmpty => must_consume = true,
				_ => abort!(
					span,
					"this attribute is not allowed on enum variants. allowed: `must_consume`"
				),
			}
		}

		Self { must_consume }
	}
}

enum ParseAttribute {
	Cut,
	With(TokenStream),
	Not(TokenStream),
	NotAfter(TokenStream),
	PostCond(PostCond),
	Longest,
	NonEmpty,
}

fn get_parse_attributes(
	attrs: &[Attribute],
) -> impl Iterator<Item = (Span, ParseAttribute)> + DoubleEndedIterator + '_ {
	attrs
		.iter()
		.filter(|attr| matches!(attr.style, AttrStyle::Outer))
		.filter_map(|attr| match attr.path.get_ident() {
			Some(ident) if ident == "parse" => {
				let stream = match attr.tokens.clone().into_iter().next().unwrap() {
					TokenTree::Group(group) => group.stream(),
					tt => abort!(
						tt.span(),
						"`parse` attributes must be followed by a parenthesized group"
					),
				};
				Some(
					Parser::parse2(Punctuated::<Meta, Comma>::parse_terminated, stream)
						.unwrap_or_else(|err| {
							abort!(
								err.span(),
								"`parse` attribute arguments must be meta separated by commas"
							)
						})
						.into_iter()
						.map(parse_meta)
						.collect(),
				)
			}
			Some(ident) if ident == "cut" => {
				if !attr.tokens.is_empty() {
					abort!(
						attr.tokens.span(),
						"no tokens are allowed after the `cut` attribute"
					);
				}
				Some(vec![(ident.span(), ParseAttribute::Cut)])
			}
			_ => None,
		})
		.flatten()
}

fn parse_meta(meta: Meta) -> (Span, ParseAttribute) {
	(
		meta.span(),
		match meta.path().get_ident() {
			Some(ident) if ident == "with" => match meta {
				Meta::NameValue(MetaNameValue {
					lit: Lit::Str(lit), ..
				}) => ParseAttribute::With(lit.parse().unwrap_or_else(|err| abort!(err.span(), err))),
				Meta::NameValue(other) => {
					abort!(other.span(), "`with` attribute takes a string argument")
				}
				other => abort!(
					other.span(),
					"`with` attribute must be a name-value attribute"
				),
			},
			Some(ident) if ident == "not" => match meta {
				Meta::NameValue(MetaNameValue {
					lit: Lit::Str(lit), ..
				}) => ParseAttribute::Not(lit.parse().unwrap_or_else(|err| abort!(err.span(), err))),
				Meta::NameValue(other) => {
					abort!(other.span(), "`not` attribute takes a string argument")
				}
				other => abort!(
					other.span(),
					"`not` attribute must be a name-value attribute"
				),
			},
			Some(ident) if ident == "not_after" => match meta {
				Meta::NameValue(MetaNameValue {
					lit: Lit::Str(lit), ..
				}) => ParseAttribute::NotAfter(lit.parse().unwrap_or_else(|err| abort!(err.span(), err))),
				Meta::NameValue(other) => {
					abort!(
						other.span(),
						"`not_after` attribute takes a string argument"
					)
				}
				other => abort!(
					other.span(),
					"`not_after` attribute must be a name-value attribute"
				),
			},
			Some(ident) if ident == "postcond" => match meta {
				Meta::List(MetaList { nested, .. }) => {
					let mut cond = None;
					let mut reason = None;

					let mut set_cond = |lit: &Lit, meta_span: Span| {
						let new_cond = match lit {
							Lit::Str(lit) => (
								lit.clone(),
								lit.parse().unwrap_or_else(|err| abort!(err.span(), err)),
							),
							_ => abort!(lit.span(), "`cond` value must be a string literal"),
						};
						if cond.replace(new_cond).is_some() {
							abort!(meta_span, "duplicate condition");
						}
					};
					let mut set_reason = |lit: &Lit, meta_span: Span| {
						let value = match lit {
							Lit::Str(lit) => lit.clone(),
							_ => abort!(lit.span(), "`reason` value must be a string literal"),
						};
						if reason.replace(value).is_some() {
							abort!(meta_span, "duplicate reason");
						}
					};

					for meta in &nested {
						match meta {
							NestedMeta::Lit(lit) => set_cond(lit, lit.span()),
							NestedMeta::Meta(meta) => match meta {
								Meta::NameValue(MetaNameValue { path, lit, .. }) => match path.get_ident() {
									Some(ident) if ident == "cond" => set_cond(lit, meta.span()),
									Some(ident) if ident == "reason" => set_reason(lit, meta.span()),
									_ => abort!(
										path.span(),
										"unknown `postcond` argument. valid are `cond` and `reason`."
									),
								},
								_ => abort!(
									meta.span(),
									"inside `postcond` only string literals and name-value metas are allowed"
								),
							},
						}
					}

					let (cond_original, cond) = cond.unwrap_or_else(|| abort!(nested.span(), "missing postcondition. use a string literal or the `cond` name-value meta to pass it."));
					ParseAttribute::PostCond(PostCond {
						cond_original,
						cond,
						reason,
					})
				}
				Meta::NameValue(MetaNameValue {
					lit: Lit::Str(lit), ..
				}) => ParseAttribute::PostCond(PostCond {
					cond_original: lit.clone(),
					cond: lit.parse().unwrap_or_else(|err| abort!(err.span(), err)),
					reason: None,
				}),
				Meta::NameValue(other) => {
					abort!(other.span(), "`postcond` attribute takes a string argument")
				}
				other => abort!(
					other.span(),
					"`postcond` attribute must be a name-value attribute"
				),
			},
			Some(ident) if ident == "longest" => match meta {
				Meta::Path(..) => ParseAttribute::Longest,
				other => abort!(other.span(), "`longest` attribute must be a path attribute"),
			},
			Some(ident) if ident == "must_consume" => match meta {
				Meta::Path(..) => ParseAttribute::NonEmpty,
				other => abort!(
					other.span(),
					"`must_consume` attribute must be a path attribute"
				),
			},
			other => abort!(
				other.span(),
				"valid attributes are `with`, `longest`, `must_consume`, `not`, `not_after`, and `postcond`"
			),
		},
	)
}
