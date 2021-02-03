#![doc(issue_tracker_base_url = "https://github.com/volllly/mer/issues/")]
#![doc(html_root_url = "https://github.com/volllly/mer")]
#![doc(test(no_crate_inject))]
//#[doc(include = "../README.md")]
//#[doc = include_str!("../../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[macro_use]
pub mod helpers;

pub mod interfaces;

#[cfg(test)]
mod test;

use anyhow::Result;

use helpers::smart_lock::SmartLock;
use log::trace;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec::Vec};

#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum Error {
  #[cfg_attr(feature = "std", error("mutex was poisoned"))]
  Lock,
  #[cfg_attr(feature = "std", error("could not register {source}: {source}"))]
  Register {
    #[cfg_attr(feature = "std", source)]
    source: anyhow::Error,
    end: String,
  },
  #[cfg_attr(feature = "std", error("{0} must be initialized"))]
  Init(String),
}

#[cfg(not(feature = "std"))]
impl From<Error> for anyhow::Error {
  fn from(e: Error) -> Self {
    anyhow::anyhow!(e)
  }
}

#[derive(Debug)]
pub struct Call<T> {
  pub procedure: String,
  pub payload: T,
}
unsafe impl<T> Send for Call<T> where T: Send {}
unsafe impl<T> Sync for Call<T> where T: Sync {}

#[derive(Debug)]
pub struct Reply<T> {
  pub payload: T,
}
unsafe impl<T> Send for Reply<T> where T: Send {}
unsafe impl<T> Sync for Reply<T> where T: Sync {}

#[derive(derive_builder::Builder)]
#[cfg_attr(not(feature = "std"), builder(no_std))]
#[builder(pattern = "owned", build_fn(skip))]
pub struct Mer<B, F>
where
  B: interfaces::Backend,
  F: interfaces::Frontend<Backend = B>,
{
  #[builder(setter(into, name = "backend_setter"), private)]
  backend: SmartLock<B>,
  #[builder(setter(into, name = "frontend_setter"), private)]
  frontend: SmartLock<F>,

  #[allow(dead_code)]
  #[builder(setter(into, name = "middlewares_setter"), private)]
  middlewares: SmartLock<Vec<Box<dyn interfaces::Middleware<Backend = B>>>>,
}

impl<'a, B, F> MerBuilder<B, F>
where
  B: interfaces::Backend,
  B: 'static,
  F: interfaces::Frontend<Backend = B>,
  F: 'static,
{
  pub fn backend(self, value: B) -> Self {
    self.backend_setter(smart_lock!(value))
  }

  pub fn frontend(self, value: F) -> Self {
    self.frontend_setter(smart_lock!(value))
  }

  pub fn middlewares(self, value: Vec<Box<dyn interfaces::Middleware<Backend = B>>>) -> Self {
    self.middlewares_setter(smart_lock!(value))
  }

  pub fn build(self) -> Result<Mer<B, F>> {
    trace!("MerBuilder.build()");

    let backend = self.backend.ok_or_else(|| Error::Init("backend".into()))?;
    let frontend = self.frontend.ok_or_else(|| Error::Init("frontend".into()))?;
    let middlewares = self.middlewares.unwrap_or_default();

    let frontend_backend = clone_lock!(frontend);
    let middlewares_backend = clone_lock!(middlewares);

    let backend_frontend = clone_lock!(backend);
    let middlewares_frontend = clone_lock!(middlewares);

    access!(backend)
      .map_err::<anyhow::Error, _>(|_| Error::Lock.into())?
      .register(move |call: Call<B::Intermediate>| {
        trace!("Mer.backend.register()");
        let middlewares_inner = access!(middlewares_backend).map_err::<anyhow::Error, _>(|_| Error::Lock.into())?;
        let unwrapped = middlewares_inner.iter().fold(Ok(call), |acc, m| m.unwrap_call(acc));

        let reply = match unwrapped {
          Ok(unwrapped_ok) => access!(frontend_backend).map_err::<anyhow::Error, _>(|_| Error::Lock.into())?.receive(unwrapped_ok),
          Err(err) => Err(err),
        };

        middlewares_inner.iter().fold(reply, |acc, m| m.wrap_reply(acc))
      })
      .map_err::<anyhow::Error, _>(|e| Error::Register { source: e, end: "backend".into() }.into())?;

    access!(frontend)
      .map_err::<anyhow::Error, _>(|_| Error::Lock.into())?
      .register(move |call: Call<B::Intermediate>| {
        trace!("Mer.frontend.register()");
        let middlewares_inner = access!(middlewares_frontend).map_err::<anyhow::Error, _>(|_| Error::Lock.into())?;

        let wrapped = middlewares_inner.iter().rev().fold(Ok(call), |acc, m| m.wrap_call(acc));

        let reply = match wrapped {
          Ok(wrapped_ok) => access!(backend_frontend).map_err::<anyhow::Error, _>(|_| Error::Lock.into())?.call(wrapped_ok),
          Err(err) => Err(err),
        };

        middlewares_inner.iter().fold(reply, |acc, m| m.unwrap_reply(acc))
      })
      .map_err::<anyhow::Error, _>(|e| Error::Register { source: e, end: "frontend".into() }.into())?;

    Ok(Mer {
      backend: clone_lock!(backend),
      frontend: clone_lock!(frontend),
      middlewares: clone_lock!(middlewares),
    })
  }
}

impl<B: interfaces::Backend, F: interfaces::Frontend<Backend = B>> Mer<B, F> {
  pub fn builder() -> MerBuilder<B, F> {
    MerBuilder::default()
  }
}

impl<'a, B: interfaces::Backend, F: interfaces::Frontend<Backend = B>> Mer<B, F> {
  pub fn start(&mut self) -> Result<()> {
    trace!("Mer.start()");
    access!(self.backend).map_err::<anyhow::Error, _>(|_| Error::Lock.into())?.start()
  }

  pub fn stop(&mut self) -> Result<()> {
    trace!("Mer.stop()");
    access!(self.backend).map_err::<anyhow::Error, _>(|_| Error::Lock.into())?.stop()
  }

  pub fn frontend<T, R>(&self, access: T) -> Result<R, Error>
  where
    T: Fn(&F) -> R,
  {
    trace!("Mer.frontend()");
    Ok(access(&*match access!(self.frontend) {
      Ok(frontend) => frontend,
      Err(_) => return Err(Error::Lock {}),
    }))
  }

  pub fn backend<T, R>(&self, access: T) -> Result<R, Error>
  where
    T: Fn(&B) -> R,
  {
    trace!("Mer.backend()");
    Ok(access(&*match access!(self.backend) {
      Ok(backend) => backend,
      Err(_) => return Err(Error::Lock {}),
    }))
  }
}
