#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::rc::Rc;
use alloc::sync::Arc;
use core::{cell::RefCell, marker::PhantomData};
use spin::Mutex;

#[cfg(test)]
mod tests;

#[cfg(feature = "backends")]
pub mod backends;
#[cfg(feature = "frontends")]
pub mod frontends;
pub mod helpers;
pub mod interfaces;

use helpers::builder::*;

#[derive(Debug)]
pub struct Error {}
unsafe impl Send for Error {}

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

pub struct Call<T> {
  pub procedure: String,
  pub payload: T,
}
unsafe impl<T> Send for Call<T> where T: Send {}

pub struct Reply<T> {
  pub payload: T,
}
unsafe impl<T> Send for Reply<T> where T: Send {}


pub struct Caller<'a, B: interfaces::Backend<'a>, F: interfaces::Frontend<'a, B>> {
  #[allow(clippy::type_complexity)]
  call: Box<dyn Fn(&Call<B::Intermediate>) -> Result<Reply<B::Intermediate>, interfaces::backend::Error> + 'a>,
  frontend: Arc<Mutex<F>>,
  backend: Arc<Mutex<B>>,
}

pub trait AutomaticCall<'a, B: interfaces::Backend<'a>> {
  #[allow(clippy::type_complexity)]
  fn call<R>(&self, call: &Call<B::Intermediate>) -> Result<Reply<R>, interfaces::backend::Error> where R: for<'de> serde::Deserialize<'de> ;
}

impl<'a, B: interfaces::Backend<'a>, F: interfaces::Frontend<'a, B>> AutomaticCall<'a, B> for Caller<'a, B, F> {
  #[allow(clippy::type_complexity)]
  fn call<R>(&self, call: &Call<B::Intermediate>) -> Result<Reply<R>, interfaces::backend::Error> where R: for<'de> serde::Deserialize<'de>  {
    let reply = (self.call)(call)?;
    Ok(Reply {
      payload: B::deserialize(&reply.payload)?,
    })
  }
}

// pub trait ManualCall<'a, B> {
//   fn manual<C>(&self, procedure: &str, payload: Box<dyn erased_serde::Serialize>) -> Result<B, backend::Error>;
// }

// impl<'a, B: backend::Backend<'a>, F: frontend::Frontend<'a>> ManualCall<'a, B> for Caller<'a, B, F> {
//   fn manual<C>(&self, procedure: &str, payload: Box<dyn erased_serde::Serialize>) -> Result<B, backend::Error> {
//     (self.call)(&Call { procedure, payload }).map(|r| r.payload)
//   }
// }

pub struct Mer<'a, B, F>
where
  B: interfaces::Backend<'a>,
  B: 'a,
  F: interfaces::Frontend<'a, B>,
  F: 'a,
{
  backend: Arc<Mutex<B>>,
  frontend: Arc<Mutex<F>>,

  #[allow(dead_code)]
  call: Caller<'a, B, F>,
}

impl<'a, B, F> Builder<B, F, Set, Set>
where
  B: interfaces::Backend<'a>,
  B: 'a,
  F: interfaces::Frontend<'a, B>,
  F: 'static,
{
  pub fn build(self) -> Mer<'a, B, F> {
    let backend = Arc::new(Mutex::new(self.backend.unwrap()));
    let frontend = Arc::new(Mutex::new(self.frontend.unwrap()));

    
    // Fn(&crate::Call<Self::Intermediate>) -> Result<crate::Reply<Box<dyn erased_serde::Serialize>>, crate::Error>
    let frontend_receiver = frontend.clone();
    backend.lock().receiver(move |call: &Call<&B::Intermediate>| {
      Ok(frontend_receiver.lock().receive(call).unwrap())
    }).unwrap();

    Mer {
      backend: backend.clone(),
      frontend: frontend.clone(),

      call: Caller {
        frontend,
        backend: backend.clone(),
        call: Box::new(move |call: &Call<B::Intermediate>| backend.lock().call(call)),
      },
    }
  }
}

impl<'a, B: interfaces::Backend<'a>, F: interfaces::Frontend<'a, B>> Mer<'a, B, F> {
  #[allow(clippy::new_ret_no_self)]
  pub fn new() -> Builder<B, F, Unset, Unset> {
    Builder {
      backend_set: PhantomData {},
      frontend_set: PhantomData {},

      backend: None,
      frontend: None,
    }
  }

  pub fn start(&mut self) -> Result<(), interfaces::backend::Error> {
    self.backend.lock().start()
  }

  pub fn stop(&mut self) -> Result<(), interfaces::backend::Error> {
    self.backend.lock().stop()
  }

  pub fn receive(&self, call: &Call<&B::Intermediate>) -> Result<Reply<B::Intermediate>, interfaces::frontend::Error> {
    self.frontend.lock().receive(call)
  }
}
