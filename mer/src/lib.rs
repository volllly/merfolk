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

use core::marker::PhantomData;

use anyhow::Result;

use log::trace;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[derive(Debug)]
pub enum Error {
  Lock {},
  MiddlewareWrappingError,
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

pub struct Mer<'a, B, F>
where
  B: interfaces::Backend,
  B: 'a,
  F: interfaces::Frontend,
  F: 'a,
{
  _phantom: PhantomData<&'a B>,

  backend: smart_lock_type!(B),
  frontend: smart_lock_type!(F),
}

pub struct MerInit<B, F> {
  pub backend: B,
  pub frontend: F,
  pub middlewares: Option<Vec<Box<dyn interfaces::Middleware<Backend = B>>>>,
}

impl<'a, B, F> MerInit<B, F>
where
  B: interfaces::Backend + 'static,
  F: interfaces::Frontend<Backend = B> + 'static,
{
  pub fn init(self) -> Mer<'a, B, F> {
    trace!("MerInit.init()");

    let backend = smart_lock!(self.backend);
    let frontend = smart_lock!(self.frontend);
    let middlewares = smart_lock!(self.middlewares.unwrap_or_default());

    let frontend_backend = clone_lock!(frontend);
    let middlewares_backend = clone_lock!(middlewares);

    let backend_frontend = clone_lock!(backend);
    let middlewares_frontend = clone_lock!(middlewares);

    access!(backend)
      .unwrap()
      .register(move |call: Call<B::Intermediate>| {
        trace!("Mer.backend.register()");
        let middlewares_inner = access!(middlewares_backend).unwrap();
        let unwrapped = middlewares_inner.iter().fold(Ok(call), |acc, m| m.unwrap_call(acc));

        let reply = match unwrapped {
          Ok(unwrapped_ok) => access!(frontend_backend).unwrap().receive(unwrapped_ok),
          Err(err) => Err(err),
        };

        middlewares_inner.iter().fold(reply, |acc, m| m.wrap_reply(acc))
      })
      .unwrap();

    access!(frontend)
      .unwrap()
      .register(move |call: Call<B::Intermediate>| {
        trace!("Mer.frontend.register()");
        let middlewares_inner = access!(middlewares_frontend).unwrap();

        let wrapped = middlewares_inner.iter().rev().fold(Ok(call), |acc, m| m.wrap_call(acc));

        let reply = match wrapped {
          Ok(wrapped_ok) => access!(backend_frontend).unwrap().call(wrapped_ok),
          Err(err) => Err(err),
        };

        middlewares_inner.iter().fold(reply, |acc, m| m.unwrap_reply(acc))
      })
      .unwrap();

    Mer {
      _phantom: PhantomData,

      backend: clone_lock!(backend),
      frontend: clone_lock!(frontend),
    }
  }
}

impl<'a, B: interfaces::Backend, F: interfaces::Frontend> Mer<'a, B, F> {
  pub fn start(&mut self) -> Result<()> {
    trace!("MerInit.start()");
    access!(self.backend).unwrap().start()
  }

  pub fn stop(&mut self) -> Result<()> {
    trace!("MerInit.stop()");
    access!(self.backend).unwrap().stop()
  }

  pub fn frontend<T, R>(&self, access: T) -> Result<R, Error>
  where
    T: Fn(&F) -> R,
  {
    trace!("MerInit.frontend()");
    Ok(access(&*match access!(self.frontend) {
      Ok(frontend) => frontend,
      Err(_) => return Err(Error::Lock {}),
    }))
  }

  pub fn backend<T, R>(&self, access: T) -> Result<R, Error>
  where
    T: Fn(&B) -> R,
  {
    trace!("MerInit.backend()");
    Ok(access(&*match access!(self.backend) {
      Ok(backend) => backend,
      Err(_) => return Err(Error::Lock {}),
    }))
  }
}
