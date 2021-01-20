use anyhow::Result;

use core::future::Future;
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

#[async_trait::async_trait(?Send)]
pub trait Backend: Send {
  type Intermediate: serde::Serialize + for<'a> serde::Deserialize<'a> + Send;

  async fn start(&mut self) -> Result<()>;
  async fn stop(&mut self) -> Result<()>;

  fn receiver<T, F>(&mut self, receiver: T) -> Result<()>
  where
    T: Fn(crate::Call<<Self as Backend>::Intermediate>) -> F + Send + Sync + 'static,
    F: Future<Output = Result<crate::Reply<<Self as Backend>::Intermediate>>>;

  async fn call(&mut self, call: crate::Call<<Self as Backend>::Intermediate>) -> Result<crate::Reply<<Self as Backend>::Intermediate>>;

  fn serialize<T: serde::Serialize>(from: &T) -> Result<<Self as Backend>::Intermediate>;

  fn deserialize<T>(from: &<Self as Backend>::Intermediate) -> Result<T>
  where
    T: for<'de> serde::Deserialize<'de>;
}
