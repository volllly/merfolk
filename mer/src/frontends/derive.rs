use interfaces::{Backend, Frontend};

use crate::interfaces;

use std::sync::{Arc, Mutex};

use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error<B: core::fmt::Display> {
  FromBackend { from: B },
  UnknownProcedure,
  MutexLock
}

impl<B: snafu::Error> From<B> for Error<B> {
  fn from(from: B) -> Self {
    Error::FromBackend { from }
  }
}

pub trait DeriveFrontend<'a, B: Backend<'a>>: Frontend<'a, B> {
  type Receiver: Receiver<'a, B>;
  type Caller: Caller<'a, B>;

  fn derive_receiver(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Error<B::Error>>;
  fn derive_caller(&mut self, caller: Arc<Mutex<Self::Caller>>);
}

pub trait Caller<'a, B: Backend<'a>>: Clone {}

pub trait Receiver<'a, B: interfaces::Backend<'a>> {
  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Error<B::Error>>;
}

