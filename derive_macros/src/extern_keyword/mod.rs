use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

pub fn expand_fn(input: &mut syn::ItemFn) -> Result<TokenStream, Vec<syn::Error>> {
  input.sig.abi = parse_quote!{ extern };

  Ok(quote! {
    #input
  })
}
