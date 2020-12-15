use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemTrait, ItemStruct, AttributeArgs};
use darling::FromMeta;

#[macro_use]
mod frontend;

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
  let compile_errors = errors.iter().map(syn::Error::to_compile_error);
  quote!(#(#compile_errors)*)
}

#[proc_macro_attribute]
pub fn frontend(args: TokenStream, input: TokenStream) -> TokenStream {
  let args = parse_macro_input!(args as AttributeArgs);

  let args_parsed = match frontend::AttrArgs::from_list(&args) {
    Ok(v) => v,
    Err(e) => { return TokenStream::from(e.write_errors()); }
  };

  if args_parsed.target.is_some() {
    let input = parse_macro_input!(input as ItemTrait);

    frontend::expand_trait(&args_parsed, &input)
    .unwrap_or_else(to_compile_errors)
    .into()
  } else {
    let input = parse_macro_input!(input as ItemStruct);

    frontend::expand_struct(&args_parsed, &input)
    .unwrap_or_else(to_compile_errors)
    .into()
  }
}
