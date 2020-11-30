use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemTrait, AttributeArgs};
use darling::FromMeta;

#[macro_use]
mod receiver;

#[proc_macro_derive(Receiver)]
pub fn derive_receiver(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  receiver::expand_struct(&input)
  .unwrap_or_else(to_compile_errors)
  .into()
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
  let compile_errors = errors.iter().map(syn::Error::to_compile_error);
  quote!(#(#compile_errors)*)
}

#[proc_macro_attribute]
pub fn receiver(args: TokenStream, input: TokenStream) -> TokenStream {
  let args = parse_macro_input!(args as AttributeArgs);
  let input = parse_macro_input!(input as ItemTrait);

  let args_parsed = match receiver::AttrArgs::from_list(&args) {
    Ok(v) => v,
    Err(e) => { return TokenStream::from(e.write_errors()); }
  };

  receiver::expand_trait(&args_parsed, &input)
  .unwrap_or_else(to_compile_errors)
  .into()
}
