#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use core::{
  marker::PhantomData,
  cell::RefCell
};
use alloc::rc::Rc;

use hashbrown::HashMap;

#[cfg(test)]
mod tests;

#[cfg(feature = "backends")]
pub mod backends;
pub mod helpers;
pub mod interfaces;

use helpers::builder::*;
use interfaces::{
  backend,
  frontend
};

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

      backend: Some(backend)
    }
  }
}

pub struct Mer<'a, T: backend::Backend<'a>> {
  #[allow(dead_code)]
  _phantom: PhantomData<&'a T>,

  backend: T,
  mapping: Rc<RefCell<HashMap<&'a str, &'a dyn frontend::Register<Intermediate = T::Intermediate>>>>
}

impl<'a, T: backend::Backend<'a>> Builder<T, Set> {
  pub fn build(self) -> Mer<'a, T> {
    let mapping = Rc::new(RefCell::new(HashMap::new()));
    let mut mer = Mer {
      _phantom: PhantomData,

      backend: self.backend.unwrap(),
      mapping: mapping.clone()
    };

    mer.backend.receiver(move |c| {
      backend::Reply {
        payload: mapping.borrow().get(c.procedure).unwrap().call(c.payload).unwrap()
      }
    });

    mer
  }
}


impl<'a, T: backend::Backend<'a>> Mer<'a, T> {
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

  pub fn register(&mut self, name: &'a str, body: &'a dyn frontend::Register<Intermediate = T::Intermediate>) -> Result<(), Error> {
    self.mapping.borrow_mut().insert(name, body);
    Ok(())
  }
}

