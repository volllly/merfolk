#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::rc::Rc;
use core::{cell::RefCell, marker::PhantomData};

use hashbrown::HashMap;

#[cfg(test)]
mod tests;

#[cfg(feature = "backends")]
pub mod backends;
#[cfg(feature = "frontends")]
pub mod frontends;
pub mod helpers;
pub mod interfaces;

use helpers::builder::*;
use interfaces::{backend, frontend};

pub struct Error {}

pub struct Builder<T, BACKEND>
where
  BACKEND: ToAssign,
{
  backend_set: PhantomData<BACKEND>,

  backend: Option<T>,
}

impl<'a, T, BACKEND> Builder<T, BACKEND>
where
  BACKEND: ToAssign,
{
  pub fn with_backend(self, backend: T) -> Builder<T, Set> {
    Builder {
      backend_set: PhantomData {},

      backend: Some(backend),
    }
  }
}


pub struct Call<'a, T> {
  pub procedure: &'a str,
  pub payload: &'a T,
}

pub struct Reply<T> {
  pub payload: T,
}

pub struct Caller<T>
where T: Fn(&Call<dyn serde::Serialize>) {
  call: T
}

pub trait Handler {
  fn handler();
}

impl<T> Handler for Caller<T>
where T: Fn(&Call<dyn serde::Serialize>) {
  fn handler() {}
}

pub struct Mer<'a, T: backend::Backend<'a>, U> {
  #[allow(dead_code)]
  _phantom: PhantomData<&'a T>,

  backend: T,

  #[allow(clippy::type_complexity)]
  mapping:
    Rc<RefCell<HashMap<&'a str, &'a dyn frontend::Procedure<'a, Intermediate = T::Intermediate>>>>,
  call: Caller<U>
}

impl<'a, T: backend::Backend<'a>, U: Fn(&Call<dyn serde::Serialize>)> Builder<T, Set> {
  pub fn build(self) -> Mer<'a, T, U> {
    let mapping = Rc::new(RefCell::new(HashMap::new()));

    let mut mer = Mer {
      _phantom: PhantomData,

      backend: self.backend.unwrap(),
      mapping: mapping.clone(),
      call: Caller {
        call: |call: &Call<dyn serde::Serialize>| {}
      },
    };

    mer
      .backend
      .receiver(move |c| Reply {
        payload: mapping
          .borrow()
          .get(c.procedure)
          .unwrap()
          .call(c.payload)
          .unwrap(),
      })
      .unwrap();

    mer
  }
}

impl<'a, T: backend::Backend<'a>, U> Mer<'a, T, U> {
  #[allow(clippy::new_ret_no_self)]
  pub fn new() -> Builder<T, Unset> {
    Builder {
      backend_set: PhantomData {},

      backend: None,
    }
  }

  pub fn start(&mut self) -> Result<(), backend::Error> {
    self.backend.start()
  }

  pub fn stop(&mut self) -> Result<(), backend::Error> {
    self.backend.stop()
  }

  pub fn register(
    &mut self,
    name: &'a str,
    body: &'a dyn frontend::Procedure<'a, Intermediate = T::Intermediate>,
  ) -> Result<(), Error> {
    self.mapping.borrow_mut().insert(name, body);
    Ok(())
  }
}
