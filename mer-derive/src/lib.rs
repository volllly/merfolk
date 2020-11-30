use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[macro_use]
mod receiver;

#[proc_macro_derive(Receiver)]
pub fn receiver(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  receiver::expand(&input)
  .unwrap_or_else(to_compile_errors)
  .into()
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
  let compile_errors = errors.iter().map(syn::Error::to_compile_error);
  quote!(#(#compile_errors)*)
}
