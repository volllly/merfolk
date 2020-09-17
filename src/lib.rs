#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

#[cfg(test)]
mod tests;

pub mod interfaces;

#[cfg(feature = "backends")]
pub mod backends;
pub mod helpers;

use helpers::builder::*;
use interfaces::backend;

pub struct Builder<T, BACKEND>
where
  BACKEND: ToAssign,
{
  backend_set: PhantomData<BACKEND>,

  backend: Option<T>,
}

impl<T, BACKEND> Builder<T, BACKEND>
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

impl<T: backend::Backend> Builder<T, Yes> {
  pub fn build(self) -> Mer<T> {
    Mer {
      backend: self.backend.unwrap(),
    }
  }
}

pub struct Mer<T: backend::Backend> {
  backend: T,
}

impl<T: backend::Backend> Mer<T> {
  #[allow(clippy::new_ret_no_self)]
  pub fn new() -> Builder<T, No> {
    Builder {
      backend_set: PhantomData {},

      backend: None,
    }
  }
}
