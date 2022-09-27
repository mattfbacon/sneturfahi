use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use proc_macro_error::abort;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
	parse_quote, Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, GenericParam, Generics,
	Lit, LitStr, Meta, MetaNameValue, TypeParamBound,
};

mod paths {
	use proc_macro2::TokenStream;
	use quote::quote;

	pub fn trait_() -> TokenStream {
		quote!(crate::parse::tree_node::TreeNode)
	}

	pub fn child() -> TokenStream {
		quote!(crate::parse::tree_node::TreeNodeChild)
	}

	pub fn location() -> TokenStream {
		quote!(crate::span::Location)
	}
}

fn assert_fn() -> TokenStream {
	let child_trait = paths::child();
	quote! {
		fn assert_tree_node_child(x: &impl #child_trait) -> &impl #child_trait {
			x
		}
	}
}

fn make_fields(fields: &Fields) -> Vec<TokenStream> {
	match fields {
		Fields::Named(fields) => fields
			.named
			.iter()
			.enumerate()
			.map(|(idx, field)| {
				let mut ident = format_ident!("field_{idx}");
				ident.set_span(field.ty.span());
				quote_spanned! {field.ty.span()=> assert_tree_node_child(#ident)}
			})
			.collect(),
		Fields::Unnamed(fields) => fields
			.unnamed
			.iter()
			.enumerate()
			.map(|(idx, field)| {
				let mut ident = format_ident!("field_{idx}");
				ident.set_span(field.ty.span());
				quote_spanned! {field.ty.span()=> assert_tree_node_child(#ident)}
			})
			.collect(),
		Fields::Unit => vec![],
	}
}

#[derive(Default)]
struct ContainerAttributes {
	name: Option<LitStr>,
	passthrough_child: bool,
}

impl ContainerAttributes {
	fn get(attrs: &[Attribute]) -> Self {
		let mut ret = Self::default();

		for attr in attrs.iter().filter(|attr| {
			attr
				.path
				.get_ident()
				.map_or(false, |ident| ident == "tree_node")
		}) {
			let inner = match attr.tokens.clone().into_iter().next() {
				Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
					group.stream()
				}
				other => abort!(other.span(), "expected parenthesized group"),
			};
			let meta: Meta = syn::parse2(inner).unwrap();
			match meta {
				Meta::Path(meta)
					if meta
						.get_ident()
						.map_or(false, |ident| ident == "passthrough_child") =>
				{
					ret.passthrough_child = true;
				}
				Meta::NameValue(MetaNameValue {
					path,
					lit: Lit::Str(name),
					..
				}) if path.get_ident().map_or(false, |ident| ident == "name") => {
					ret.name = Some(name);
				}
				other => abort!(other.span(), "expected passthrough_child or name"),
			}
		}

		ret
	}
}

pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut input = syn::parse_macro_input!(input as DeriveInput);

	let attrs = ContainerAttributes::get(&input.attrs);

	add_trait_bounds(&mut input.generics);

	let name = input.ident;
	let name_str = name.to_string();

	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let trait_path = paths::trait_();
	let child_path = paths::child();
	let location_path = paths::location();
	let assert_fn = assert_fn();

	let experimental_impl = implement(&input.data, |fields| {
		quote! {
			#assert_fn
			false #(|| #child_path::experimental(#fields))*
		}
	});

	let start_location_impl = implement(&input.data, |fields| {
		quote! {
			#assert_fn
			#(
				if let Some(location) = #child_path::start_location(#fields) {
					return Some(location);
				}
			)*
			None
		}
	});

	let end_location_impl = implement(&input.data, |fields| {
		let fields = fields.iter().rev();
		quote! {
			#assert_fn
			#(
				if let Some(location) = #child_path::end_location(#fields) {
					return Some(location);
				}
			)*
			None
		}
	});

	let for_each_child_impl = implement(&input.data, |fields| {
		quote! {
			#assert_fn
			#(
				#child_path::invoke_with_self(#fields, f);
			)*
		}
	});

	if attrs.passthrough_child {
		quote! {
			#[automatically_derived]
			impl #impl_generics #child_path for #name #ty_generics #where_clause {
				fn invoke_with_self<'a>(&'a self, f: &mut dyn FnMut(&'a dyn #trait_path)) {
					#for_each_child_impl
				}

				fn experimental(&self) -> bool {
					#experimental_impl
				}

				fn start_location(&self) -> Option<#location_path> {
					#start_location_impl
				}

				fn end_location(&self) -> Option<#location_path> {
					#end_location_impl
				}
			}

			#[automatically_derived]
			impl #impl_generics #child_path for Box<#name #ty_generics> #where_clause {
				fn invoke_with_self<'a>(&'a self, f: &mut dyn FnMut(&'a dyn #trait_path)) {
					<#name #ty_generics>::invoke_with_self(self, f);
				}

				fn experimental(&self) -> bool {
					<#name #ty_generics>::experimental(self)
				}

				fn start_location(&self) -> Option<#location_path> {
					<#name #ty_generics>::start_location(self)
				}

				fn end_location(&self) -> Option<#location_path> {
					<#name #ty_generics>::end_location(self)
				}
			}
		}
		.into()
	} else {
		let name_str = attrs
			.name
			.unwrap_or_else(|| LitStr::new(&name_str, Span::call_site()));
		quote! {
			#[automatically_derived]
			impl #impl_generics #trait_path for #name #ty_generics #where_clause {
				fn name(&self) -> &'static str {
					#name_str
				}

				fn experimental(&self) -> bool {
					#experimental_impl
				}

				fn start_location(&self) -> Option<#location_path> {
					#start_location_impl
				}

				fn end_location(&self) -> Option<#location_path> {
					#end_location_impl
				}

				fn for_each_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn #trait_path)) {
					#for_each_child_impl
				}
			}
		}
		.into()
	}
}

fn implement(input: &Data, struct_fn: impl FnMut(&[TokenStream]) -> TokenStream) -> TokenStream {
	match &input {
		Data::Struct(data) => implement_struct(data, struct_fn),
		Data::Enum(data) => implement_enum(data, struct_fn),
		Data::Union(union) => abort!(
			union.union_token.span(),
			"`TreeNode` can only be derived on structs and enums"
		),
	}
}

fn make_destructure(fields: &Fields) -> TokenStream {
	match fields {
		Fields::Named(fields) => {
			let names = fields.named.iter().enumerate().map(|(idx, field)| {
				let theirs = &field.ident;
				let ours = format_ident!("field_{idx}");
				quote!(#theirs: #ours)
			});
			quote!({#(#names),*})
		}
		Fields::Unnamed(fields) => {
			let names = (0..fields.unnamed.len()).map(|idx| format_ident!("field_{idx}"));
			quote!((#(#names),*))
		}
		Fields::Unit => quote!(),
	}
}

fn implement_struct(
	data: &DataStruct,
	mut struct_fn: impl FnMut(&[TokenStream]) -> TokenStream,
) -> TokenStream {
	let body = struct_fn(&make_fields(&data.fields));
	let destructure = make_destructure(&data.fields);
	quote! {
		let Self #destructure = self;
		#body
	}
}

fn implement_enum(
	data: &DataEnum,
	mut struct_fn: impl FnMut(&[TokenStream]) -> TokenStream,
) -> TokenStream {
	let patterns = data.variants.iter().map(|variant| {
		let ident = &variant.ident;
		let destructure = make_destructure(&variant.fields);
		quote!(Self::#ident #destructure)
	});
	let arms = data
		.variants
		.iter()
		.map(|variant| struct_fn(&make_fields(&variant.fields)));

	quote! {
		match self {
			#(#patterns => {
				#arms
			})*
		}
	}
}

fn add_trait_bounds(generics: &mut Generics) {
	let child_path = paths::child();
	let bound: TypeParamBound = parse_quote!(#child_path);

	for param in &mut generics.params {
		if let GenericParam::Type(type_param) = param {
			type_param.bounds.push(bound.clone());
		}
	}
}
