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

#[derive(Debug)]
pub enum Error {
  Lock {},
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

    let frontend_receiver = clone_lock!(frontend);
    let backend_caller = clone_lock!(backend);

    access_mut!(backend)
      .unwrap()
      .receiver(move |call: Call<B::Intermediate>| {
        trace!("Mer.backend.receiver()");

        Ok(access!(frontend_receiver).unwrap().receive(call).unwrap())
      }) //TODO: fix error
      .unwrap();

    access_mut!(frontend)
      .unwrap()
      .caller(move |call: Call<B::Intermediate>| {
        trace!("Mer.frontend.caller()");

        access!(backend_caller).unwrap().call(call)
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
