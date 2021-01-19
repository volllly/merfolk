use anyhow::Result;

pub trait Backend: Send {
  type Intermediate: serde::Serialize + for<'a> serde::Deserialize<'a>;

  fn start(&mut self) -> Result<()>;
  fn stop(&mut self) -> Result<()>;

  fn receiver<T>(&mut self, receiver: T) -> Result<()>
  where
    T: Fn(crate::Call<<Self as Backend>::Intermediate>) -> Result<crate::Reply<<Self as Backend>::Intermediate>> + Send + Sync + 'static;

  fn call(&mut self, call: crate::Call<<Self as Backend>::Intermediate>) -> Result<crate::Reply<<Self as Backend>::Intermediate>>;

  fn serialize<T: serde::Serialize>(from: &T) -> Result<<Self as Backend>::Intermediate>;

  fn deserialize<T>(from: &<Self as Backend>::Intermediate) -> Result<T>
  where
    T: for<'de> serde::Deserialize<'de>;
}
