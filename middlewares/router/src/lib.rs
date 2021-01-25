use std::marker::PhantomData;

use mer::{
  interfaces::{Backend, Middleware},
  Call, Reply,
};

use anyhow::Result;
use thiserror::Error;

use log::trace;
use regex::Regex;
#[derive(Debug, Error)]
pub enum Error {
  #[error("could not parse regex: {0}")]
  RegexParse(#[from] regex::Error)
}

pub struct Router<B> {
  __phantom: PhantomData<B>,

  routes: Vec<(String, String)>,
}

#[derive(Default)]
pub struct RouterInit {
  pub routes: Vec<(String, String)>,
}

impl RouterInit {
  pub fn init<B>(self) -> Router<B> {
    trace!("initialze RouterInit");

    Router {
      __phantom: PhantomData,

      routes: self.routes,
    }
  }

  pub fn init_boxed<B>(self) -> Box<Router<B>> {
    trace!("initialze RouterInit");

    Box::new(self.init())
  }
}

impl<B: Backend> Middleware for Router<B> {
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
}
