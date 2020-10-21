#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::rc::Rc;
use core::{cell::RefCell, marker::PhantomData};

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

pub struct Builder<T, F, BACKEND, FRONTEND>
where
  BACKEND: ToAssign,
  FRONTEND: ToAssign,
{
  backend_set: PhantomData<BACKEND>,
  frontend_set: PhantomData<FRONTEND>,

  backend: Option<T>,
  frontend: Option<F>,
}

impl<'a, T, F, BACKEND, FRONTEND> Builder<T, F, BACKEND, FRONTEND>
where
  BACKEND: ToAssign,
  FRONTEND: ToAssign,
{
  pub fn with_backend(self, backend: T) -> Builder<T, F, Set, FRONTEND> {
    Builder {
      backend_set: PhantomData {},
      frontend_set: self.frontend_set,

      backend: Some(backend),
      frontend: self.frontend,
    }
  }

  pub fn with_frontnd(self, frontend: F) -> Builder<T, F, BACKEND, Set> {
    Builder {
      backend_set: self.backend_set,
      frontend_set: PhantomData {},

      backend: self.backend,
      frontend: Some(frontend),
    }
  }
}

pub struct Call<'a, T> {
  pub procedure: &'a str,
  pub payload: T,
}

pub struct Reply<T> {
  pub payload: T,
}

pub struct Caller<'a> {
  call: Rc<dyn Fn(&Call<Box<dyn erased_serde::Serialize>>) + 'a>,
}

pub trait Handler<'a, T> {
  fn handler(&self) -> Rc<dyn Fn(&Call<T>) + 'a>;
}

impl<'a> Handler<'a, Box<dyn erased_serde::Serialize>> for Caller<'a> {
  fn handler(&self) -> Rc<dyn Fn(&Call<Box<dyn erased_serde::Serialize>>) + 'a> {
    self.call.clone()
  }
}

pub struct Mer<'a, T, F>
where
  T: backend::Backend<'a>,
  T: 'a,
  F: frontend::Frontend<'a>,
  F: 'a,
{
  #[allow(dead_code)]
  _phantom: PhantomData<T>,

  backend: Rc<RefCell<T>>,
  frontend: Rc<RefCell<F>>,

  #[allow(clippy::type_complexity)]
  call: Caller<'a>,
}

impl<'a, T, F> Builder<T, F, Set, Set>
where
  T: backend::Backend<'a>,
  T: 'a,
  F: frontend::Frontend<'a>,
  F: 'a,
{
  pub fn build(self) -> Mer<'a, T, F> {
    let backend = Rc::new(RefCell::new(self.backend.unwrap()));
    let frontend = Rc::new(RefCell::new(self.frontend.unwrap()));

    Mer {
      _phantom: PhantomData,

      backend: backend.clone(),
      frontend: frontend.clone(),

      call: Caller {
        call: Rc::new(move |call: &Call<Box<dyn erased_serde::Serialize>>| {
          backend.borrow_mut().call(call).unwrap();
        }),
      },
    }
  }
}

impl<'a, T: backend::Backend<'a>, F: frontend::Frontend<'a>> Mer<'a, T, F> {
  #[allow(clippy::new_ret_no_self)]
  pub fn new() -> Builder<T, F, Unset, Unset> {
    Builder {
      backend_set: PhantomData {},
      frontend_set: PhantomData {},

      backend: None,
      frontend: None,
    }
  }

  pub fn start(&mut self) -> Result<(), backend::Error> {
    self.backend.borrow_mut().start()
  }

  pub fn stop(&mut self) -> Result<(), backend::Error> {
    self.backend.borrow_mut().stop()
  }

  pub fn receive(
    &self,
    call: &Call<&F::Intermediate>,
  ) -> Result<Reply<Box<dyn erased_serde::Serialize>>, frontend::Error> {
    self.frontend.borrow().receive(call)
  }

  // pub fn register<C>(&mut self, name: &'a str, body: C) -> Result<(), Error>
  // where
  //   C: Fn(&crate::Call<F::Intermediate>) -> Result<crate::Reply<F::Intermediate>, Error>,
  //   C: 'a,
  // {
  //   self.mapping.borrow_mut().insert(name, Box::new(body));
  //   Ok(())
  // }
}
