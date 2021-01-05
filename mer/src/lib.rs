#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[macro_use]
pub mod helpers;

#[cfg(feature = "backends")]
pub mod backends;
#[cfg(feature = "frontends")]
pub mod frontends;

pub mod interfaces;

use core::marker::PhantomData;

use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
  Lock {}
}

pub struct Call<T> {
  pub procedure: String,
  pub payload: T,
}
unsafe impl<T> Send for Call<T> where T: Send {}

pub struct Reply<T> {
  pub payload: T,
}
unsafe impl<T> Send for Reply<T> where T: Send {}

pub struct Mer<'a, B, F>
where
  B: interfaces::Backend<'a>,
  B: 'a,
  F: interfaces::Frontend<'a, B>,
  F: 'a,
{
  _phantom: PhantomData<&'a B>,

  backend: smart_lock_type!(B),
  frontend: smart_lock_type!(F),
  // call: Option<F::Call>,
}

pub struct MerInit<B, F> {
  pub backend: B,
  pub frontend: F,
}

impl<'a, B, F> MerInit<B, F>
where
  B: interfaces::Backend<'a>,
  F: interfaces::Frontend<'a, B>,
{
  pub fn init(self) -> Mer<'a, B, F> {
    let backend = smart_lock!(self.backend);
    let frontend = smart_lock!(self.frontend);

    let frontend_receiver = clone_lock!(frontend);
    let backend_caller = clone_lock!(backend);

    access_mut!(backend)
      .unwrap()
      .receiver(smart_pointer!(move |call: &Call<&B::Intermediate>| {
        Ok(access!(frontend_receiver).unwrap().receive(call).unwrap())
      })) //TODO: fix error
      .unwrap();

    access_mut!(frontend)
      .unwrap()
      .caller(smart_pointer!(move |call: &Call<&B::Intermediate>| {
        access!(backend_caller).unwrap().call(call)
      }))
      .unwrap();

    Mer {
      _phantom: PhantomData,

      backend: clone_lock!(backend),
      frontend: clone_lock!(frontend),
    }
  }
}

impl<'a, B: interfaces::Backend<'a>, F: interfaces::Frontend<'a, B>> Mer<'a, B, F> {
  pub fn start(&mut self) -> Result<(), B::Error> {
    access!(self.backend).unwrap().start()
  }

  pub fn stop(&mut self) -> Result<(), B::Error> {
    access!(self.backend).unwrap().stop()
  }

  pub fn frontend<T, R>(&self, access: T) -> Result<R, Error> where T: Fn(&F) -> R {
    Ok(access(&*match access!(self.frontend) {
      Ok(frontend) => frontend,
      Err(_) => return Err(Error::Lock {})
    }))
  }

  pub fn backend<T, R>(&self, access: T) -> Result<R, Error> where T: Fn(&B) -> R {
    Ok(access(&*match access!(self.backend) {
      Ok(backend) => backend,
      Err(_) => return Err(Error::Lock {})
    }))
  }

  pub fn frontend_clone(&self) -> smart_lock_type!(F) {
    clone_lock!(self.frontend)
  }

  pub fn backend_clone(&self) -> smart_lock_type!(F) {
    clone_lock!(self.frontend)
  }
}
