use std::marker::PhantomData;

use mer::{
  interfaces::{Backend, Middleware},
  Call, Reply,
};

use anyhow::Result;
use thiserror::Error;

use regex::Regex;
#[derive(Debug, Error)]
pub enum Error {
  #[error("could not parse regex: {0}")]
  RegexParse(#[from] regex::Error),
}

#[derive(derive_builder::Builder)]
#[builder(pattern = "owned")]
pub struct Router<B> {
  #[builder(private, default = "PhantomData")]
  __phantom: PhantomData<B>,

  #[builder(setter(into), default = "vec![]")]
  routes: Vec<(String, String)>,
}

impl<B> RouterBuilder<B> {
  pub fn build_boxed(self) -> std::result::Result<Box<Router<B>>, RouterBuilderError> {
    self.build().map(Box::new)
  }
}

impl<B> Router<B> {
  pub fn builder() -> RouterBuilder<B> {
    RouterBuilder::default()
  }
}

impl<B: Backend + 'static> Middleware for Router<B> {
  type Backend = B;

  fn wrap_call(&self, call: Result<crate::Call<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Call<<Self::Backend as Backend>::Intermediate>> {
    call
  }

  fn wrap_reply(&self, reply: Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>> {
    reply
  }

  fn unwrap_call(&self, mut call: Result<crate::Call<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Call<<Self::Backend as Backend>::Intermediate>> {
    if let Ok(call_ok) = &mut call {
      let route = self.routes.iter().find_map(|r| match Regex::new(&r.0) {
        Err(err) => Some(Err(err)),
        Ok(regex) => {
          if regex.is_match(&call_ok.procedure) {
            Some(Ok(regex.replace_all(&call_ok.procedure, r.1.as_str())))
          } else {
            None
          }
        }
      });

      if let Some(route) = route {
        call_ok.procedure = route.map_err(Error::RegexParse)?.to_string();
      }
    }
    call
  }

  fn unwrap_reply(&self, reply: Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>> {
    reply
  }

  fn as_any(&mut self) -> &mut dyn core::any::Any {
    self
  }
}
