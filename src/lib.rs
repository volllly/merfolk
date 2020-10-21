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

pub struct Builder<B, F, BACKEND, FRONTEND>
where
  BACKEND: ToAssign,
  FRONTEND: ToAssign,
{
  backend_set: PhantomData<BACKEND>,
  frontend_set: PhantomData<FRONTEND>,

  backend: Option<B>,
  frontend: Option<F>,
}

impl<'a, B, F, BACKEND, FRONTEND> Builder<B, F, BACKEND, FRONTEND>
where
  BACKEND: ToAssign,
  FRONTEND: ToAssign,
{
  pub fn with_backend(self, backend: B) -> Builder<B, F, Set, FRONTEND> {
    Builder {
      backend_set: PhantomData {},
      frontend_set: self.frontend_set,

      backend: Some(backend),
      frontend: self.frontend,
    }
  }

  pub fn with_frontnd(self, frontend: F) -> Builder<B, F, BACKEND, Set> {
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

pub struct Caller<'a, B: backend::Backend<'a>, F: frontend::Frontend<'a>> {
  #[allow(clippy::type_complexity)]
  call: Rc<dyn Fn(&Call<Box<dyn erased_serde::Serialize>>) -> Result<Reply<B::Intermediate>, backend::Error> + 'a>,
  frontend: Rc<RefCell<F>>,
  backend: Rc<RefCell<B>>,
}

pub trait AutomaticCall<'a> {
  #[allow(clippy::type_complexity)]
  fn call<R: serde::Deserialize<'a>>(&self, call: &Call<Box<dyn erased_serde::Serialize>>) -> Result<Reply<R>, backend::Error>;
}

impl<'a, B: backend::Backend<'a>, F: frontend::Frontend<'a>> AutomaticCall<'a> for Caller<'a, B, F> {
  #[allow(clippy::type_complexity)]
  fn call<R: serde::Deserialize<'a>>(&self, call: &Call<Box<dyn erased_serde::Serialize>>) -> Result<Reply<R>, backend::Error> {
    let reply = (self.call)(call)?;

    Ok(Reply {
      payload: self.backend.borrow().deserialize(&reply.payload)?,
    })
  }
}

pub trait ManualCall<'a, B> {
  fn manual<C>(&self, procedure: &str, payload: Box<dyn erased_serde::Serialize>) -> Result<B, backend::Error>;
}

impl<'a, B: backend::Backend<'a>, F: frontend::Frontend<'a>> ManualCall<'a, B> for Caller<'a, B, F> {
  fn manual<C>(&self, procedure: &str, payload: Box<dyn erased_serde::Serialize>) -> Result<B, backend::Error> {
    self.handler()(&Call { procedure, payload }).map(|r| r.payload)
  }
}

pub struct Mer<'a, B, F>
where
  B: backend::Backend<'a>,
  B: 'a,
  F: frontend::Frontend<'a>,
  F: 'a,
{
  backend: Rc<RefCell<B>>,
  frontend: Rc<RefCell<F>>,

  #[allow(dead_code)]
  call: Caller<'a, B, F>,
}

impl<'a, B, F> Builder<B, F, Set, Set>
where
  B: backend::Backend<'a>,
  B: 'a,
  F: frontend::Frontend<'a>,
  F: 'a,
{
  pub fn build(self) -> Mer<'a, B, F> {
    let backend = Rc::new(RefCell::new(self.backend.unwrap()));
    let frontend = Rc::new(RefCell::new(self.frontend.unwrap()));

    Mer {
      backend,
      frontend,

      call: Caller {
        call: Rc::new(move |call: &Call<Box<dyn erased_serde::Serialize>>| backend.borrow_mut().call(call)),
        frontend,
        backend,
      },
    }
  }
}

impl<'a, B: backend::Backend<'a>, F: frontend::Frontend<'a>> Mer<'a, B, F> {
  #[allow(clippy::new_ret_no_self)]
  pub fn new() -> Builder<B, F, Unset, Unset> {
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

  pub fn receive(&self, call: &Call<&F::Intermediate>) -> Result<Reply<Box<dyn erased_serde::Serialize>>, frontend::Error> {
    self.frontend.borrow().receive(call)
  }
}
