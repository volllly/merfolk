use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

#[derive(Debug, FromMeta)]
pub struct AttrArgs {
  #[darling(default)]
  pub target: Option<syn::Path>,
}

pub fn expand_trait(args: &AttrArgs, input: &syn::ItemTrait) -> Result<TokenStream, Vec<syn::Error>> {
  let trait_name = &input.ident;
  let trait_generics = &input.generics;
  let service_name = args.target.as_ref().unwrap();
  let where_clause = &trait_generics.where_clause;

  let items = &input.items;
  let item_methods: Vec<syn::TraitItemMethod> = items
    .iter()
    .filter_map(|i| match i {
      syn::TraitItem::Method(m) => {
        let mut method = m.clone();
        method.default.take();
        Some(method)
      }
      _ => None,
    })
    .collect();

  let mut impl_generic_def = trait_generics.clone();
  impl_generic_def.params.insert(0, syn::parse_quote! { __B: mer::interfaces::Backend<'__a> });
  impl_generic_def.params.insert(0, syn::parse_quote! { '__a });

  let mut impl_generics = trait_generics.clone();
  impl_generics.params.insert(0, syn::parse_quote! { __B });
  impl_generics.params.insert(0, syn::parse_quote! { '__a });

  let receiver_impl_items: Vec<TokenStream> = item_methods
    .iter()
    .map(|i| {
      let item_name = format_ident!("{}", &i.sig.ident);
      let mut has_self: bool = false;
      let arguments: Vec<&Box<syn::Type>> = i
        .sig
        .inputs
        .iter()
        .filter_map(|a| match a {
          syn::FnArg::Typed(t) => Some(&t.ty),
          syn::FnArg::Receiver(_) => {
            has_self = true;
            None
          }
        })
        .collect();
      let index = (0..arguments.len()).map(syn::Index::from);

      let deser = quote! {
        let deser_payload = __B::deserialize::<(#( #arguments ),*)>(call.payload)?;
      };

      let reply = match has_self {
        true => quote! {
          let reply = <Self as #trait_name #impl_generics>::#item_name(self, #( deser_payload.#index),*);
        },
        false => quote! {
          let reply = <Self as #trait_name #impl_generics>::#item_name(#( deser_payload.#index),*);
        },
      };

      let ser = quote! {
        let ser_reply = __B::serialize(&reply)?;
        Ok(mer::Reply { payload: ser_reply })
      };

      quote! {
        stringify!(#item_name) => {
          log::debug!("frontend procedure receiving: {}", stringify!(#item_name));

          #deser
          #reply
          #ser
        }
      }
    })
    .collect();

  let caller_impl_items: Vec<TokenStream> = item_methods
    .iter()
    .map(|i| {
      let item_name = &i.sig.ident;
      let mut has_self = false;

      let arguments: Vec<&syn::Ident> = i
        .sig
        .inputs
        .iter()
        .filter_map(|a| match a {
          syn::FnArg::Typed(t) => {
            if let syn::Pat::Ident(ident) = &*t.pat {
              Some(&ident.ident)
            } else {
              None
            }
          }
          syn::FnArg::Receiver(_) => {
            has_self = true;
            None
          }
        })
        .collect();

      let mut signature = i.sig.inputs.clone();
      if !has_self {
        signature.insert(0, syn::parse_quote! { &self })
      }
      let old_return_type: syn::Type = match &i.sig.output {
        syn::ReturnType::Default => syn::parse_quote! { () },
        syn::ReturnType::Type(_, t) => syn::parse_quote! { #t },
      };
      let return_type: syn::ReturnType = syn::parse_quote! { -> Result<#old_return_type, mer_frontend_derive::Error<__B::Error>> };

      quote! {
        pub fn #item_name(#signature) #return_type {
          log::debug!("frontend procedure calling: {}", stringify!(#item_name));

          let ser_payload = __B::serialize(&(#( #arguments ),*))?;

          let reply = self.__call.as_ref().unwrap()(&mer::Call {
            procedure: stringify!(#item_name).to_string(),
            payload: &ser_payload,
          })?
          .payload;

          let deser_reply = __B::deserialize::<#old_return_type>(&reply);
          Ok(deser_reply?)
        }
      }
    })
    .collect();

  Ok(quote! {
    trait #trait_name #impl_generic_def #where_clause {
      #( #item_methods )*
    }

    impl #impl_generic_def #trait_name #impl_generics for #service_name #impl_generics #where_clause {
      #( #items )*
    }

    impl #impl_generic_def #service_name #impl_generics #where_clause {
        fn __receive(&self, call: &mer::Call<&__B::Intermediate>) -> Result<mer::Reply<__B::Intermediate>, mer_frontend_derive::Error<__B::Error>> {
          match call.procedure.as_str() {
            #( #receiver_impl_items ),*
            _ => Err(mer_frontend_derive::Error::UnknownProcedure {}),
          }
        }
    }


    impl #impl_generic_def #service_name #impl_generics #where_clause {
      #( #caller_impl_items )*
    }
  })
}

pub fn expand_struct(input: &syn::ItemStruct) -> Result<TokenStream, Vec<syn::Error>> {
  let struct_name = &input.ident;
  let struct_name_init = format_ident!("{}Init", &input.ident);
  let struct_generics = &input.generics;
  let where_clause = &struct_generics.where_clause;

  let fields = match &input.fields {
    syn::Fields::Named(named) => {
      let fields: Vec<&syn::Field> = named.named.iter().collect();
      quote! { #( #fields ),* }
    }
    rest => quote! { #rest },
  };

  let field_names = match &input.fields {
    syn::Fields::Named(named) => named.named.iter().filter_map(|f| f.ident.clone()).map(|f| quote! { #f: self.#f }).collect(),
    _ => vec![],
  };

  let mut impl_generic_def = struct_generics.clone();
  impl_generic_def.params.insert(0, syn::parse_quote! { __B: mer::interfaces::Backend<'__a> });
  impl_generic_def.params.insert(0, syn::parse_quote! { '__a });

  let mut impl_generics = struct_generics.clone();
  impl_generics.params.insert(0, syn::parse_quote! { __B });
  impl_generics.params.insert(0, syn::parse_quote! { '__a });

  Ok(quote! {
    struct #struct_name #impl_generic_def #where_clause {
      #fields,

      __call: Option<Box<dyn Fn(&mer::Call<&__B::Intermediate>) -> Result<mer::Reply<__B::Intermediate>, __B::Error> + '__a + Send>>
    }

    #[derive(Default)]
    struct #struct_name_init #struct_generics #where_clause {
      #fields,
    }

    impl #struct_generics #struct_name_init #struct_generics #where_clause {
      pub fn init<'__a, __B: mer::interfaces::Backend<'__a>>(self) -> #struct_name #impl_generics {
        #struct_name {
          #( #field_names ),*,

          __call: None
        }
      }
    }

    impl #impl_generic_def mer::interfaces::Frontend<'__a, __B> for #struct_name #impl_generics #where_clause {
      type Intermediate = String;
      type Error = mer_frontend_derive::Error<__B::Error>;

      fn caller<__T>(&mut self, caller: __T) -> Result<(), mer_frontend_derive::Error<__B::Error>>
      where
        __T: Fn(&mer::Call<&__B::Intermediate>) -> Result<mer::Reply<__B::Intermediate>, __B::Error> + '__a + Send,
        __T: 'static,
      {
        self.__call = Some(Box::new(caller));
        Ok(())
      }

      fn receive(&self, call: &mer::Call<&__B::Intermediate>) -> Result<mer::Reply<__B::Intermediate>, mer_frontend_derive::Error<__B::Error>> {
        log::debug!("receiving: Call {{ prodecure: {:?}, payload: ... }}", &call.procedure);

        self.__receive(call).map_err(core::convert::Into::into)
      }
    }
  })
}