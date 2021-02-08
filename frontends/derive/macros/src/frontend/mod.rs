use darling::{util::Flag, FromMeta};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

#[derive(Debug, FromMeta)]
pub struct Args {
  #[darling(default)]
  pub target: Option<syn::Path>,
  #[darling(default)]
  pub definition_only: Flag,
}

pub fn expand_trait(args: &Args, input: &syn::ItemTrait) -> Result<TokenStream, Vec<syn::Error>> {
  let trait_name = &input.ident;
  let trait_generics = &input.generics;
  let service_name = args.target.as_ref().unwrap();
  let where_clause = &trait_generics.where_clause;

  let items = &input.items;
  let filtered_items: Vec<&syn::TraitItem> = input.items.iter().filter(|i| {
    match &i {
      syn::TraitItem::Method(m) => {
        if let Some(attr) = m.attrs.iter().find(|a| a.path.segments.last().map_or(false, |p| p.ident == "frontend")) {
          match Args::from_meta(&if let Ok(meta) = attr.parse_meta() { meta } else { return true; }) {
            Ok(a) => {
              a.definition_only.is_none()
            },
            Err(_) => {
              true
            }
          }
        } else {
          true
        }
      }
      _ => true,
    }
  }).collect();
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
  impl_generic_def.params.insert(0, syn::parse_quote! { __B: ::mer_frontend_derive::reexports::mer::interfaces::Backend });
  impl_generic_def.params.insert(0, syn::parse_quote! { '__a });

  let mut impl_generics = trait_generics.clone();
  impl_generics.params.insert(0, syn::parse_quote! { __B });
  impl_generics.params.insert(0, syn::parse_quote! { '__a });
  
  let filtered_item_methods: Vec<&syn::TraitItemMethod> = item_methods
  .iter()
  .filter(|i| {
    if let Some(attr) = i.attrs.iter().find(|a| a.path.segments.last().map_or(false, |p| p.ident == "frontend")) {
      match Args::from_meta(&if let Ok(meta) = attr.parse_meta() { meta } else { return true; }) {
        Ok(a) => {
          a.definition_only.is_none()
        },
        Err(_) => {
          true
        }
      }
    } else {
      true
    }
  }).collect();

  let receiver_impl_items: Vec<TokenStream> = filtered_item_methods
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
        let deser_payload = __B::deserialize::<(#( #arguments ),*)>(&call.payload)?;
      };

      let reply = if arguments.len() != 1 {
        match has_self {
          true => quote! {
            let reply = <Self as #trait_name #impl_generics>::#item_name(self, #( deser_payload.#index),*);
          },
          false => quote! {
            let reply = <Self as #trait_name #impl_generics>::#item_name(#( deser_payload.#index),*);
          },
        }
      } else {
        match has_self {
          true => quote! {
            let reply = <Self as #trait_name #impl_generics>::#item_name(self, deser_payload);
          },
          false => quote! {
            let reply = <Self as #trait_name #impl_generics>::#item_name(deser_payload);
          },
        }
      };

      let ser = quote! {
        let ser_reply = __B::serialize(&reply)?;
        Ok(::mer_frontend_derive::reexports::mer::Reply { payload: ser_reply })
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
      let return_type: syn::ReturnType = syn::parse_quote! { -> ::mer_frontend_derive::reexports::anyhow::Result<#old_return_type> };

      quote! {
        pub fn #item_name(#signature) #return_type {
          log::debug!("frontend procedure calling: {}", stringify!(#item_name));

          let ser_payload = __B::serialize(&(#( #arguments ),*))?;

          let reply = self.__call.as_ref().unwrap()(::mer_frontend_derive::reexports::mer::Call {
            procedure: stringify!(#item_name).to_string(),
            payload: ser_payload,
          })?
          .payload;

          let deser_reply = __B::deserialize::<#old_return_type>(&reply);
          Ok(deser_reply?)
        }
      }
    })
    .collect();

  #[cfg(feature = "std")]
  let error = quote! { Err(mer_frontend_derive::Error::UnknownProcedure { procedure: call.procedure }.into()) };
  #[cfg(not(feature = "std"))]
  let error = quote! { Err(::mer_frontend_derive::reexports::anyhow::anyhow!("unknown procedure: {}", call.procedure)) };

  Ok(quote! {
    trait #trait_name #impl_generic_def #where_clause {
      #( #filtered_item_methods )*
    }

    impl #impl_generic_def #trait_name #impl_generics for #service_name #impl_generics #where_clause {
      #( #filtered_items )*
    }

    impl #impl_generic_def #service_name #impl_generics #where_clause {
        fn __receive(&self, call: ::mer_frontend_derive::reexports::mer::Call<__B::Intermediate>) -> ::mer_frontend_derive::reexports::anyhow::Result<::mer_frontend_derive::reexports::mer::Reply<__B::Intermediate>> {
          match call.procedure.as_str() {
            #( #receiver_impl_items ),*
            _ => {
              #error
            },
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
  let struct_name_builder = format_ident!("{}Builder", &input.ident);
  let struct_generics = &input.generics;
  let where_clause = &struct_generics.where_clause;

  let fields = match &input.fields {
    syn::Fields::Named(named) => {
      let fields: Vec<&syn::Field> = named.named.iter().collect();
      quote! { #( #fields ),* }
    }
    rest => quote! { #rest },
  };

  let mut impl_generic_def = struct_generics.clone();
  impl_generic_def.params.insert(0, syn::parse_quote! { __B: ::mer_frontend_derive::reexports::mer::interfaces::Backend });
  impl_generic_def.params.insert(0, syn::parse_quote! { '__a });

  let mut impl_generics = struct_generics.clone();
  impl_generics.params.insert(0, syn::parse_quote! { __B });
  impl_generics.params.insert(0, syn::parse_quote! { '__a });

  Ok(quote! {
    #[derive(::mer_frontend_derive::reexports::derive_builder::Builder)]
    #[builder(pattern = "owned")]
    #[cfg_attr(not(feature = "std"), builder(no_std))]
    struct #struct_name #impl_generic_def #where_clause {
      #[builder(private, default = "None")]
      __call: Option<Box<dyn Fn(::mer_frontend_derive::reexports::mer::Call<__B::Intermediate>) -> ::mer_frontend_derive::reexports::anyhow::Result<::mer_frontend_derive::reexports::mer::Reply<__B::Intermediate>> + '__a + Send>>,

      #fields
    }


    impl #impl_generic_def #struct_name #impl_generics #where_clause {
      pub fn builder() -> #struct_name_builder #impl_generics {
        #struct_name_builder::default()
      }
    }

    impl #impl_generic_def ::mer_frontend_derive::reexports::mer::interfaces::Frontend for #struct_name #impl_generics #where_clause {
      type Backend = __B;

      fn register<__T>(&mut self, caller: __T) -> ::mer_frontend_derive::reexports::anyhow::Result<()>
      where
        __T: Fn(::mer_frontend_derive::reexports::mer::Call<__B::Intermediate>) -> ::mer_frontend_derive::reexports::anyhow::Result<::mer_frontend_derive::reexports::mer::Reply<__B::Intermediate>> + '__a + Send,
      {
        self.__call = Some(Box::new(caller));
        Ok(())
      }

      fn receive(&self, call: ::mer_frontend_derive::reexports::mer::Call<__B::Intermediate>) -> ::mer_frontend_derive::reexports::anyhow::Result<::mer_frontend_derive::reexports::mer::Reply<__B::Intermediate>> {
        log::debug!("receiving: Call {{ procedure: {:?}, payload: ... }}", &call.procedure);

        self.__receive(call).map_err(::core::convert::Into::into)
      }
    }
  })
}
