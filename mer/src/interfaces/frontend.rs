use crate::interfaces::Backend;

use anyhow::Result;

use core::future::Future;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

#[async_trait::async_trait(?Send)]
pub trait Frontend: Send {
  type Backend: Backend;

  fn caller<T, F>(&mut self, caller: T) -> Result<()>
  where
    T: Fn(crate::Call<<Self::Backend as Backend>::Intermediate>) -> F + Send + Sync + 'static,
    F: Future<Output = Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>>;

  async fn receive(&self, call: crate::Call<<Self::Backend as Backend>::Intermediate>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>;
}
