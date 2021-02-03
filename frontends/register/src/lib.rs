use mer::{
  interfaces::{Backend, Frontend},
  Call, Reply,
};

use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use anyhow::Result;
use thiserror::Error;

use log::trace;

#[derive(Debug, Error)]
pub enum Error {
  #[error("backend error: {0}")]
  FromBackend(#[from] anyhow::Error),
  #[error("called procedure is not registered: {0}")]
  ProcedureNotRegistered(String),
  #[error("procedures lock was poinsoned")]
  Lock,
  #[error("call not registered mer init()")]
  CallNotRegistered,
}

#[derive(derive_builder::Builder)]
#[builder(pattern = "owned")]
pub struct Register<'a, B: Backend> {
  #[allow(clippy::type_complexity)]
  #[builder(setter(name = "procedures_setter"), private, default = "Arc::new(Mutex::new(HashMap::new()))")]
  procedures: Arc<Mutex<HashMap<String, Box<dyn Fn(Call<B::Intermediate>) -> Result<Reply<B::Intermediate>> + 'a>>>>,

  #[allow(clippy::type_complexity)]
  #[builder(private, default = "None")]
  call: Option<Box<dyn Fn(Call<B::Intermediate>) -> Result<Reply<B::Intermediate>> + 'a + Send>>,
}

impl<'a, B: Backend> RegisterBuilder<'a, B> {
  pub fn procedures<P, C: for<'de> serde::Deserialize<'de>, R: serde::Serialize>(self, value: HashMap<&'a str, P>) -> Self
  where
    P: Fn(C) -> R + 'a,
  {
    let mut map: HashMap<String, Box<dyn Fn(Call<B::Intermediate>) -> Result<Reply<B::Intermediate>> + 'a>> = HashMap::new();

    value.into_iter().for_each(|p| {
      map.insert(
        p.0.to_string(),
        Box::new(move |call: Call<B::Intermediate>| {
          let reply = p.1(B::deserialize::<C>(&call.payload)?);
          Ok(Reply { payload: B::serialize::<R>(&reply)? })
        }),
      );
    });

    self.procedures_setter(Arc::new(Mutex::new(map)))
  }
}

impl<'a, B: Backend> Register<'a, B> {
  pub fn builder() -> RegisterBuilder<'a, B> {
    RegisterBuilder::default()
  }
}

unsafe impl<'a, T: Backend> Send for Register<'a, T> {}

impl<'a, B: Backend> Register<'a, B> {
  pub fn register<P, C: for<'de> serde::Deserialize<'de>, R: serde::Serialize>(&self, name: &str, procedure: P) -> Result<()>
  where
    P: Fn(C) -> R + 'a,
  {
    trace!("register procedure");

    self.procedures.lock().map_err(|_| Error::Lock)?.insert(
      name.to_string(),
      Box::new(move |call: Call<B::Intermediate>| {
        let reply = procedure(B::deserialize::<C>(&call.payload)?);
        Ok(Reply { payload: B::serialize::<R>(&reply)? })
      }),
    );
    Ok(())
  }

  pub fn call<C: serde::Serialize, R: for<'de> serde::Deserialize<'de>>(&self, procedure: &str, payload: &C) -> Result<R> {
    trace!("call procedure");

    Ok(B::deserialize(
      &self.call.as_ref().ok_or(Error::CallNotRegistered)?(Call {
        procedure: procedure.to_string(),
        payload: B::serialize(&payload)?,
      })?
      .payload,
    )?)
  }
}

impl<'a, B: Backend> Frontend for Register<'a, B> {
  type Backend = B;

  fn register<T>(&mut self, caller: T) -> Result<()>
  where
    T: Fn(Call<<Self::Backend as Backend>::Intermediate>) -> Result<Reply<<Self::Backend as Backend>::Intermediate>> + 'a + Send,
  {
    trace!("register caller");

    self.call = Some(Box::new(caller));
    Ok(())
  }

  #[allow(clippy::type_complexity)]
  fn receive(&self, call: Call<<Self::Backend as Backend>::Intermediate>) -> Result<Reply<<Self::Backend as Backend>::Intermediate>> {
    trace!("receive call");

    self
      .procedures
      .lock()
      .map_err(|_| Error::Lock)?
      .get(&call.procedure)
      .ok_or_else::<anyhow::Error, _>(|| Error::ProcedureNotRegistered(call.procedure.to_owned()).into())?(call)
  }
}
