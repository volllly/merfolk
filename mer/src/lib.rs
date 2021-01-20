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
  pub async fn init(self) -> Mer<'a, B, F> {
    trace!("MerInit.init()");

    let backend = smart_lock!(self.backend);
    let frontend = smart_lock!(self.frontend);

    let frontend_receiver = clone_lock!(frontend);
    let backend_caller = clone_lock!(backend);

    access!(backend)
      .await
      .receiver(move |call: Call<B::Intermediate>| {
        let frontend_receiver = clone_lock!(frontend_receiver);
        async move {
          trace!("Mer.backend.receiver()");

          access!(frontend_receiver).await.receive(call).await
        }
      }) //TODO: fix error
      .unwrap();

    access!(frontend)
      .await
      .caller(move |call: Call<B::Intermediate>| {
        let backend_caller = clone_lock!(backend_caller);

        async move {
          trace!("Mer.frontend.caller()");

          access!(backend_caller).await.call(call).await
        }
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
  pub async fn start(&mut self) -> Result<()> {
    trace!("MerInit.start()");
    access!(self.backend).await.start().await
  }

  pub async fn stop(&mut self) -> Result<()> {
    trace!("MerInit.stop()");
    access!(self.backend).await.stop().await
  }

  pub async fn frontend<T, R>(&self, access: T) -> Result<R, Error>
  where
    T: Fn(&F) -> R,
  {
    trace!("MerInit.frontend()");
    Ok(access(&*access!(self.frontend).await))
  }

  pub async fn backend<T, R>(&self, access: T) -> Result<R, Error>
  where
    T: Fn(&B) -> R,
  {
    trace!("MerInit.backend()");
    Ok(access(&*access!(self.backend).await))
  }
}
