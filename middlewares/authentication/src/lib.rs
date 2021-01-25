use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use mer::{
  interfaces::{Backend, Middleware},
  Call, Reply,
};

use anyhow::Result;
use thiserror::Error;

use log::trace;
use wildmatch::WildMatch;

#[derive(Debug, Error)]
pub enum Error {
  #[error("could not find scope for procedure: {procedure}")]
  ScopeNotFound { procedure: String },
  #[error("authentication failed for {auth} in scope {scope:?}: {source}")]
  AuthenticationFailed {
    auth: String,
    scope: Option<String>,
    #[source]
    source: anyhow::Error,
  },
  #[error("no vaildator was provided during init()")]
  NoValidatorRegistered,
}

pub struct Authentication<'a, B> {
  __phantom: PhantomData<B>,

  scopes: Option<Vec<(String, String)>>,

  #[allow(clippy::type_complexity)]
  authenticator: Option<Box<dyn Fn((String, String), Vec<String>) -> Result<()> + 'a + Send>>,

  auth: (String, String),
}

#[derive(Default)]
pub struct AuthenticationInit<'a> {
  pub scopes: Option<Vec<(String, String)>>,

  #[allow(clippy::type_complexity)]
  pub authenticator: Option<Box<dyn Fn((String, String), Vec<String>) -> Result<()> + 'a + Send>>,

  pub auth: (String, String),
}

impl<'a> AuthenticationInit<'a> {
  pub fn init<B>(self) -> Authentication<'a, B> {
    trace!("initialze AuthenticationInit");

    Authentication {
      __phantom: PhantomData,

      authenticator: self.authenticator,
      scopes: self.scopes,
      auth: self.auth,
    }
  }

  pub fn init_boxed<B>(self) -> Box<Authentication<'a, B>> {
    trace!("initialze AuthenticationInit");

    Box::new(self.init())
  }
}

#[derive(Serialize, Deserialize)]
struct Intermediate<B: Backend> {
  auth: (String, String),
  payload: B::Intermediate,
}

impl<'a, B: Backend> Middleware for Authentication<'a, B> {
  type Backend = B;

  fn wrap_call(&self, call: Result<crate::Call<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Call<<Self::Backend as Backend>::Intermediate>> {
    if let Ok(call) = call {
      Ok(Call {
        procedure: call.procedure,
        payload: B::serialize(&Intermediate::<B> {
          auth: (self.auth.0.to_string(), self.auth.1.to_string()),
          payload: call.payload,
        })?,
      })
    } else {
      call
    }
  }

  fn wrap_reply(&self, reply: Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>> {
    reply
  }

  fn unwrap_call(&self, call: Result<crate::Call<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Call<<Self::Backend as Backend>::Intermediate>> {
    if let Ok(call) = call {
      let mut scope: Vec<String> = vec![];

      if let Some(scopes) = &self.scopes {
        scopes.iter().for_each(|s| {
          if WildMatch::new(&s.0).is_match(call.procedure.as_str()) {
            scope.push(s.1.to_string())
          };
        });
      }

      let intermediate: Intermediate<B> = B::deserialize(&call.payload)?;

      if let Err(err) = self.authenticator.as_ref().ok_or::<Error>(Error::NoValidatorRegistered)?(intermediate.auth, scope) {
        Err(err)
      } else {
        Ok(Call {
          procedure: call.procedure,
          payload: intermediate.payload,
        })
      }
    } else {
      call
    }
  }

  fn unwrap_reply(&self, reply: Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>> {
    reply
  }
}
