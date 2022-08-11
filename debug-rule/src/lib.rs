use quote::quote;
use syn::{parse_macro_input, ItemFn};

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
