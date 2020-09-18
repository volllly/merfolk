#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use core::marker::PhantomData;

#[cfg(test)]
mod tests;

#[cfg(feature = "backends")]
pub mod backends;
pub mod helpers;
pub mod interfaces;

use helpers::builder::*;
use interfaces::backend;

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
  pub fn with_backend(self, backend: T) -> Builder<T, Yes> {
    Builder {
      backend_set: PhantomData {},

      backend: Some(backend),
    }
  }
}

impl<'a, T: backend::Backend<'a>> Builder<T, Yes> {
  pub fn build(self) -> Mer<'a, T> {
    Mer {
      backend: self.backend.unwrap(),
      _phantom: PhantomData
    }
  }
}

pub struct Mer<'a, T: backend::Backend<'a>> {
  #[allow(dead_code)]
  backend: T,
  _phantom: PhantomData<&'a T>,
}

impl<'a, T: backend::Backend<'a>> Mer<'a, T> {
  #[allow(clippy::new_ret_no_self)]
  pub fn new() -> Builder<T, No> {
    Builder {
      backend_set: PhantomData {},

      backend: None,
    }
  }
}
