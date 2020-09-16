use std::marker::PhantomData;
use std::fmt::Debug;

#[cfg(test)]
mod tests;

pub mod interfaces;
pub mod backends;

#[derive(Debug)]
pub struct Yes;
#[derive(Debug)]
pub struct No;

impl ToAssign for Yes {}
impl ToAssign for No {}

impl Assigned for Yes {}
impl NotAssigned for No {}

pub trait ToAssign: Debug {}
pub trait Assigned: ToAssign {}
pub trait NotAssigned: ToAssign {}

pub struct Builder<T, BACKEND>
  where BACKEND: ToAssign {
    backend_set: PhantomData<BACKEND>,

    backend: Option<T>
}

impl<T, BACKEND> Builder<T, BACKEND>
  where BACKEND: ToAssign {

    pub fn with_backend(self, backend: T) -> Builder<T, Yes> {
      Builder {
        backend_set: PhantomData {},

        backend: Some(backend)
      }
    }
}

impl<T> Builder<T, Yes>
  where T: interfaces::Backend {
  pub fn build(self) -> Mer<T> {
    Mer {
      backend: self.backend.unwrap()
    }
  }
}


pub struct Mer<T: interfaces::Backend> {
  backend: T
}

impl<T> Mer<T>
  where T: interfaces::Backend {

  #[allow(clippy::new_ret_no_self)]
  pub fn new() -> Builder<T, No> {
    Builder {
      backend_set: PhantomData {},

      backend: None
    }
  }
}
