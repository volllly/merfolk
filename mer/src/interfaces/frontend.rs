use crate::interfaces::Backend;

use anyhow::Result;

pub trait Frontend: Send {
  type Backend: Backend;
  type Intermediate: serde::Serialize + for<'a> serde::Deserialize<'a>;

  fn caller<T>(&mut self, caller: T) -> Result<()>
  where
    T: Fn(crate::Call<<Self::Backend as Backend>::Intermediate>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>> + Send + Sync + 'static;

  fn receive(&self, call: crate::Call<<Self::Backend as Backend>::Intermediate>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>;
}
