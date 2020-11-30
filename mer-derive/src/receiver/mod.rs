use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn expand(input: &syn::DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
  Ok(quote! {

  })
}
