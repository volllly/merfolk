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

use helpers::smart_pointer::*;

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

  backend: SmartPointer<B>,
  frontend: SmartPointer<F>,
  // call: Option<F::Call>,
}

pub struct MerInit<B, F> {
  pub backend: B,
  pub frontend: F,
}

impl<'a, B, F> MerInit<B, F>
where
  B: interfaces::Backend<'a>,
  B: 'static,
  F: interfaces::Frontend<'a, B>,
  F: 'static,
{
  pub fn init(self) -> Mer<'a, B, F> {
    let backend = smart_pointer!(self.backend);
    let frontend = smart_pointer!(self.frontend);

    let frontend_receiver = clone!(frontend);
    let backend_caller = clone!(backend);

    access_mut!(backend)
      .unwrap()
      .receiver(move |call: &Call<&B::Intermediate>| Ok(access!(frontend_receiver).unwrap().receive(call).unwrap()))
      .unwrap();

    access_mut!(frontend)
      .unwrap()
      .caller(move |call: &Call<&B::Intermediate>| {
        access_mut!(backend_caller).unwrap().call(call)
      })
      .unwrap();

    Mer {
      _phantom: PhantomData,

      backend: clone!(backend),
      frontend: clone!(frontend),
    }
  }
}

impl<'a, B: interfaces::Backend<'a>, F: interfaces::Frontend<'a, B>> Mer<'a, B, F> {
  pub fn start(&mut self) -> Result<(), B::Error> {
    access_mut!(self.backend).unwrap().start()
  }

  pub fn stop(&mut self) -> Result<(), B::Error> {
    access_mut!(self.backend).unwrap().stop()
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

  pub fn frontend_clone(&self) -> SmartPointer<F> {
    clone!(self.frontend)
  }

  pub fn backend_clone(&self) -> SmartPointer<F> {
    clone!(self.frontend)
  }
}
