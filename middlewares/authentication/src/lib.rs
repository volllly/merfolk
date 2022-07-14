use std::marker::PhantomData;

use anyhow::Result;
use merfolk::{
  interfaces::{Backend, Middleware},
  Call, Reply,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wildmatch::WildMatch;

#[derive(Debug, Error)]
pub enum Error {
  #[error("could not find scope for procedure: {procedure}")]
  ScopeNotFound { procedure: String },
  #[error("auth was not provided")]
  NoAuth,
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

#[derive(derive_builder::Builder)]
#[builder(pattern = "owned")]
pub struct Authentication<'a, B> {
  #[builder(private, default = "PhantomData")]
  __phantom: PhantomData<B>,

  #[builder(setter(into, strip_option), default = "None")]
  scopes: Option<Vec<(String, String)>>,

  #[allow(clippy::type_complexity)]
  #[builder(setter(name = "authenticator_setter", strip_option), private, default = "None")]
  authenticator: Option<Box<dyn Fn((String, String), Vec<String>) -> Result<()> + 'a + Send>>,

  #[builder(setter(into, strip_option), default = "None")]
  auth: Option<(String, String)>,
}

impl<'a, B> AuthenticationBuilder<'a, B> {
  pub fn authenticator<A>(self, value: A) -> Self
  where
    A: Fn((String, String), Vec<String>) -> Result<()> + 'a + Send,
  {
    self.authenticator_setter(Box::new(value))
  }

  pub fn build_boxed(self) -> std::result::Result<Box<Authentication<'a, B>>, AuthenticationBuilderError> {
    self.build().map(Box::new)
  }
}

impl<'a, B> Authentication<'a, B> {
  pub fn builder() -> AuthenticationBuilder<'a, B> {
    AuthenticationBuilder::default()
  }
}

#[derive(Serialize, Deserialize)]
struct Intermediate<B: Backend> {
  auth: (String, String),
  payload: B::Intermediate,
}

impl<B: Backend + 'static> Middleware for Authentication<'static, B> {
  type Backend = B;

  fn wrap_call(&self, call: Result<crate::Call<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Call<<Self::Backend as Backend>::Intermediate>> {
    let auth = self.auth.as_ref().ok_or(Error::NoAuth)?;

    if let Ok(call) = call {
      Ok(Call {
        procedure: call.procedure,
        payload: B::serialize(&Intermediate::<B> {
          auth: (auth.0.to_string(), auth.1.to_string()),
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
          if WildMatch::new(&s.0).matches(call.procedure.as_str()) {
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

  fn as_any(&mut self) -> &mut dyn core::any::Any {
    self
  }
}
