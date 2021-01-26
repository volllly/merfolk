use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[macro_use]
mod extern_keyword;

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
  let compile_errors = errors.iter().map(syn::Error::to_compile_error);
  quote!(#(#compile_errors)*)
}

#[proc_macro_attribute]
pub fn extern_keyword(_args: TokenStream, input: TokenStream) -> TokenStream {
  let mut input = parse_macro_input!(input as ItemFn);

  extern_keyword::expand_fn(&mut input).unwrap_or_else(to_compile_errors).into()
}
