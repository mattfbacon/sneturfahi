#![deny(
	absolute_paths_not_starting_with_crate,
	elided_lifetimes_in_paths,
	explicit_outlives_requirements,
	future_incompatible,
	keyword_idents,
	macro_use_extern_crate,
	meta_variable_misuse,
	missing_abi,
	non_ascii_idents,
	nonstandard_style,
	noop_method_call,
	pointer_structural_match,
	private_in_public,
	rust_2018_idioms,
	unused_qualifications
)]
#![warn(
	clippy::pedantic,
	missing_copy_implementations,
	missing_debug_implementations
)]
#![allow(
	clippy::tabs_in_doc_comments, // rustfmt formats our doc comments and we use tabs
	clippy::redundant_else, // sometimes it's clearer
)]
#![forbid(unsafe_code)]

use proc_macro2::{Span, TokenStream, TokenTree};
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::token::Comma;
use syn::{
	parse_macro_input, parse_quote, AttrStyle, Attribute, Data, DeriveInput, Field, Fields,
	GenericParam, Generics, Ident, Index, ItemFn, Lit, LitStr, Meta, MetaList, MetaNameValue,
	NestedMeta,
};

#[proc_macro_attribute]
pub fn debug_rule(
	_attrs: proc_macro::TokenStream,
	body: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	let ItemFn {
		attrs,
		vis,
		sig,
		block,
	} = parse_macro_input!(body as ItemFn);
	let name = syn::LitStr::new(&sig.ident.to_string(), proc_macro::Span::call_site().into());
	quote! {
		#(#attrs)*
		#vis #sig {
			debug_rule_start!(#name);
			let result = ((|input: &str| -> ParseResult<'_> #block) as for<'a> fn(&'a str) -> ParseResult<'a>)(input);
			debug_rule_end!(#name, result);
			result
		}
	}
	.into()
}

#[proc_macro_error]
#[proc_macro_derive(Parse, attributes(cut, parse))]
pub fn derive_parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;
	add_trait_bounds(&mut input.generics);
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let body = implement_parse(&input.data, &input.attrs, &name);

	quote! {
		#[automatically_derived]
		impl #impl_generics Parse for #name #ty_generics #where_clause {
			fn parse(input: &[crate::lex::Token]) -> crate::parse::ParseResult<'_, Self> {
				#body
			}
		}
	}
	.into()
}

fn add_trait_bounds(generics: &mut Generics) {
	for param in &mut generics.params {
		if let GenericParam::Type(type_param) = param {
			type_param.bounds.push(parse_quote!(Parse));
		}
	}
}

fn implement_parse(data: &Data, attrs: &[Attribute], ident: &Ident) -> TokenStream {
	let attrs = ContainerAttributes::get(attrs);

	let inner = if let Some(with) = attrs.with {
		quote_spanned!(with.span()=> nom::Parser::parse(&mut #with, input))
	} else {
		match data {
			Data::Struct(data) => implement_struct(&data.fields, ident, quote!(Self)),
			Data::Enum(data) => {
				if data.variants.is_empty() {
					abort!(
						data.brace_token.span,
						"at least one field is required. maybe you wanted a unit struct?"
					);
				}

				let branches = data.variants.iter().map(|variant| {
					if matches!(variant.fields, Fields::Unit) {
						abort!(variant.span(), "enum unit variants are not allowed");
					}
					let variant_ident = &variant.ident;
					let body = implement_struct(
						&variant.fields,
						ident,
						quote_spanned!(variant_ident.span()=> Self::#variant_ident),
					);
					quote_spanned! {variant.span()=>
						|input| #body
					}
				});

				quote! {
					nom::branch::alt((
						#(#branches,)*
					))(input)
				}
			}
			Data::Union(union) => abort!(
				union.union_token.span(),
				"`Parse` can only be derived on structs and enums"
			),
		}
	};

	if attrs.post_conds.is_empty() {
		inner
	} else {
		let check_post_conds = attrs.post_conds.iter().map(
			|PostCond {
			   cond_original,
			   cond,
			   reason,
			 }| {
				let reason = reason
					.as_ref()
					.cloned()
					.unwrap_or_else(|| LitStr::new(&cond_original.value(), Span::call_site()));
				quote! {
					if !((#cond) as fn(&Self) -> bool)(&value) {
						return Err(
							nom::Err::Error(
								crate::parse::error::WithLocation {
									location: input,
									error: crate::parse::error::Error::PostConditionFailed(#reason),
								}
							)
						);
					}
				}
			},
		);

		quote!({
			let (rest, value): (_, Self) = #inner?;

			#(#check_post_conds)*

			Ok((rest, value))
		})
	}
}

fn implement_struct(fields: &Fields, ident: &Ident, constructor: TokenStream) -> TokenStream {
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
				.map(|(idx, field)| {
					let ident = format_ident!("field_{idx}");
					quote_spanned!(field.span()=> #ident)
				})
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
		abort!(
			fields.span(),
			"at least one field is required. maybe you wanted a unit struct/variant?"
		);
	}

	let fields = if named {
		quote!({ #(#field_names,)* })
	} else {
		quote!((#(#field_names,)*))
	};

	quote! {
		nom::combinator::map(nom::sequence::tuple((
			#(#elements,)*
		)), |(#(#field_names,)*)| #constructor #fields)(input)
	}
}

fn make_elements(i: &Punctuated<Field, Comma>) -> impl Iterator<Item = TokenStream> + '_ {
	i.iter().map(|field| {
		let attrs = FieldAttributes::get(&field.attrs);
		let ty_span = field.ty.span();

		let inner = attrs.with.unwrap_or_else(|| quote!(Parse::parse));

		let actual = if attrs.cut {
			quote_spanned! {ty_span=> nom::combinator::cut(#inner) }
		} else {
			quote_spanned! {ty_span=> #inner }
		};

		let nots = attrs.nots;
		if !nots.is_empty() {
			let idx = Index::from(nots.len());
			quote! {
				nom::combinator::map(
					nom::sequence::tuple((
						#(nom::combinator::not(<#nots as Parse>::parse),)*
						#actual,
					)),
					|tuple| tuple.#idx
				)
			}
		} else {
			actual
		}
	})
}

struct ContainerAttributes {
	with: Option<TokenStream>,
	post_conds: Vec<PostCond>,
}

impl ContainerAttributes {
	fn get(attrs: &[Attribute]) -> Self {
		let mut ret = Self {
			with: None,
			post_conds: Vec::new(),
		};

		for (span, attr) in get_parse_attributes(attrs) {
			match attr {
				ParseAttribute::Not(..) => abort!(span, "`not` attribute can only be used on fields. maybe you meant to put the `not` on the first field?"),
				ParseAttribute::Cut => abort!(span, "`cut` attribute can only be used on fields. maybe you meant to put the `cut` on the first field?"),
				ParseAttribute::PostCond(post_cond) => ret.post_conds.push(post_cond),
				ParseAttribute::With(with) => if ret.with.replace(with).is_some() {
					abort!(span, "multiple `with` attributes are not allowed");
				}
			}
		}

		ret
	}
}

struct FieldAttributes {
	cut: bool,
	nots: Vec<TokenStream>,
	with: Option<TokenStream>,
}

impl FieldAttributes {
	fn get(attrs: &[Attribute]) -> Self {
		let mut nots = Vec::new();
		let mut cut = false;
		let mut with = None;

		for (span, attr) in get_parse_attributes(attrs) {
			match attr {
				ParseAttribute::Not(not) => nots.push(not),
				ParseAttribute::Cut => cut = true,
				ParseAttribute::PostCond(_) => {
					abort!(span, "`postcond` attribute is only allowed on containers")
				}
				ParseAttribute::With(attr) => {
					if with.replace(attr).is_some() {
						abort!(span, "multiple `with` attributes are not allowed")
					}
				}
			}
		}

		Self { with, nots, cut }
	}
}

struct PostCond {
	cond_original: LitStr,
	cond: TokenStream,
	reason: Option<LitStr>,
}

enum ParseAttribute {
	Cut,
	With(TokenStream),
	Not(TokenStream),
	PostCond(PostCond),
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
			other => abort!(
				other.span(),
				"valid attributes are `with`, `not`, and `postcond`"
			),
		},
	)
}
