use proc_macro2::TokenStream;
use quote::quote;
use darling::FromMeta;

pub fn expand_struct(input: &syn::DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
  Ok(quote! {

  })
}


#[derive(Debug, FromMeta)]
pub struct AttrArgs {
    data: syn::Path,
}

pub fn expand_trait(args: &AttrArgs, input: &syn::ItemTrait) -> Result<TokenStream, Vec<syn::Error>> {
  let trait_name = &input.ident;
  let trait_generics = &input.generics;
  let data_name = &args.data;
  let where_clause = &trait_generics.where_clause;

  let items = &input.items;
  let item_methods: Vec<syn::TraitItemMethod> = items.iter().filter_map(|i| match i {
    syn::TraitItem::Method(m) => {
      let mut method = m.clone();
      method.default.take();
      Some(method)
    },
    _ => None
  }).collect();

  let mut receiver_impl_generics = trait_generics.clone();
  receiver_impl_generics.params.insert(0, syn::parse_str("__B: mer::interfaces::Backend<'__a>").unwrap());
  receiver_impl_generics.params.insert(0, syn::parse_str("'__a").unwrap());

  let impl_items: Vec<TokenStream> = item_methods.iter().map(|i| {
    let item_name = &i.sig.ident;
    let mut has_self: bool = false;
    let arguments: Vec<&Box<syn::Type>> = i.sig.inputs.iter().filter_map(|a| {
      match a {
        syn::FnArg::Typed(t) => Some(&t.ty),
        syn::FnArg::Receiver(_) => {
          has_self = true;
          None
        },
      }
    }).collect();
    let index = (0..arguments.len()).map(syn::Index::from);

    let deser = quote! {
      let deser_payload = __B::deserialize::<(#( #arguments ),*)>(call.payload)?;
    };

    let reply = match has_self {
      true => quote! {
        let reply = self.#item_name(#( deser_payload.#index),*);
      },
      false => quote! {
        let reply = Self::#item_name(#( deser_payload.#index),*);
      }
    };

    let ser = quote! {
      let ser_reply = __B::serialize(&reply)?;
      Ok(mer::Reply { payload: ser_reply })
    };

    quote! {
      stringify!(#item_name) => {
        #deser
        #reply
        #ser
      }
    }
  }).collect();

  Ok(quote! {
    trait #trait_name #trait_generics #where_clause {
      #( #item_methods )*
    }

    impl #trait_generics #trait_name #trait_generics for #data_name #trait_generics #where_clause {
      #( #items )*
    }

    impl #receiver_impl_generics mer::frontends::derive::Receiver<'__a, __B>
      for #data_name #trait_generics #where_clause {
        fn receive(&self, call: &mer::Call<&__B::Intermediate>) -> Result<mer::Reply<__B::Intermediate>, mer::frontends::derive::Error<__B::Error>> {
          match call.procedure.as_str() {
            #( #impl_items ),*
            _ => Err(mer::frontends::derive::Error::UnknownProcedure {}),
          }
        }
    }
  })
}
