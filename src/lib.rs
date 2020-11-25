#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(not(feature = "threadsafe"))]
use alloc::rc::Rc;
use alloc::sync::Arc;
#[cfg(not(feature = "threadsafe"))]
use core::cell::RefCell;

use core::marker::PhantomData;

#[cfg(not(feature = "std"))]
use spin::Mutex;
#[cfg(feature = "std")]
use std::sync::Mutex;

use snafu::{ResultExt, Snafu};

#[cfg(test)]
mod tests;

#[cfg(feature = "backends")]
pub mod backends;
#[cfg(feature = "frontends")]
pub mod frontends;
pub mod helpers;
pub mod interfaces;

use helpers::builder::*;

#[derive(Debug, Snafu)]
pub enum Error {
  MakeCall { source: interfaces::backend::Error },
  ReceiveCall { source: interfaces::frontend::Error },
  Start { source: interfaces::backend::Error },
  Stop { source: interfaces::backend::Error },
}
pub type Result<T, E = Error> = core::result::Result<T, E>;

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

  pub fn with_frontend(self, frontend: F) -> Builder<B, F, BACKEND, Set> {
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

#[cfg(feature = "threadsafe")]
struct SmartPointer<T>(Arc<Mutex<T>>);
#[cfg(not(feature = "threadsafe"))]
struct SmartPointer<T>(Rc<RefCell<T>>);
unsafe impl<T> Send for SmartPointer<T> {}

#[cfg(feature = "threadsafe")]
macro_rules! smart_pointer {
  ($x:expr) => {
    SmartPointer(Arc::new(Mutex::new($x)))
  };
}

#[cfg(not(feature = "threadsafe"))]
macro_rules! smart_pointer {
  ($x:expr) => {
    SmartPointer(Rc::new(RefCell::new($x)))
  };
}

#[cfg(feature = "threadsafe")]
macro_rules! clone {
  ($x:expr) => {
    SmartPointer($x.0.clone())
  };
}

#[cfg(all(feature = "threadsafe", feature = "std"))]
macro_rules! access {
  ($x:expr) => {
    $x.0.lock()
  };
}

#[cfg(all(feature = "threadsafe", not(feature = "std")))]
macro_rules! access {
  ($x:expr) => {
    Ok(*$x.0.lock())
  };
}

#[cfg(feature = "threadsafe")]
macro_rules! access_mut {
  ($x:expr) => {
    access!($x)
  };
}

#[cfg(not(feature = "threadsafe"))]
macro_rules! clone {
  ($x:expr) => {
    SmartPointer($x.0.clone())
  };
}

#[cfg(not(feature = "threadsafe"))]
macro_rules! access {
  ($x:expr) => {
    $x.0.borrow()
  };
}

#[cfg(not(feature = "threadsafe"))]
macro_rules! access_mut {
  ($x:expr) => {
    $x.0.borrow_mut()
  };
}

pub struct Caller<'a, B: interfaces::Backend<'a>> {
  #[allow(clippy::type_complexity)]
  call: Box<dyn Fn(&Call<&B::Intermediate>) -> Result<Reply<B::Intermediate>> + 'a>,
}

pub trait AutomaticCall<'a, B: interfaces::Backend<'a>> {
  #[allow(clippy::type_complexity)]
  fn call<R>(&self, call: &Call<&B::Intermediate>) -> Result<Reply<R>>
  where
    R: for<'de> serde::Deserialize<'de>;
}

impl<'a, B: interfaces::Backend<'a>> AutomaticCall<'a, B> for Caller<'a, B> {
  #[allow(clippy::type_complexity)]
  fn call<R>(&self, call: &Call<&B::Intermediate>) -> Result<Reply<R>>
  where
    R: for<'de> serde::Deserialize<'de>,
  {
    let reply = (self.call)(call)?;
    Ok(Reply {
      payload: B::deserialize(&reply.payload).context(MakeCall)?,
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
  backend: SmartPointer<B>,
  frontend: SmartPointer<F>,

  #[allow(dead_code)]
  call: Caller<'a, B>,
}

impl<'a, B, F> Builder<B, F, Set, Set>
where
  B: interfaces::Backend<'a>,
  B: 'a,
  F: interfaces::Frontend<'a, B>,
  F: 'static,
{
  pub fn build(self) -> Mer<'a, B, F> {
    let backend = smart_pointer!(self.backend.unwrap());
    let frontend = smart_pointer!(self.frontend.unwrap());

    let frontend_receiver = clone!(frontend);
    access_mut!(backend)
      .unwrap()
      .receiver(move |call: &Call<&B::Intermediate>| Ok(access!(frontend_receiver).unwrap().receive(call).unwrap()))
      .unwrap();

    Mer {
      backend: clone!(backend),
      frontend,

      call: Caller {
        call: Box::new(move |call: &Call<&B::Intermediate>| access_mut!(backend).unwrap().call(call).context(MakeCall)),
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

  pub fn start(&mut self) -> Result<()> {
    access_mut!(self.backend).unwrap().start().context(Start)
  }

  pub fn stop(&mut self) -> Result<()> {
    access_mut!(self.backend).unwrap().stop().context(Stop)
  }

  pub fn receive(&self, call: &Call<&B::Intermediate>) -> Result<Reply<B::Intermediate>> {
    access_mut!(self.frontend).unwrap().receive(call).context(ReceiveCall)
  }
}
