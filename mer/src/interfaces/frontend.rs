use super::Backend;

use anyhow::Result;

pub trait Frontend: Send {
  type Backend: Backend;

  fn register<T>(&mut self, caller: T) -> Result<()>
  where
    T: Fn(crate::Call<<Self::Backend as Backend>::Intermediate>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>> + Send + Sync + 'static;

  fn receive(&self, call: crate::Call<<Self::Backend as Backend>::Intermediate>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>;
}
