use crate::interfaces::Backend;

pub trait Frontend: Send {
  type Backend: Backend;
  type Intermediate: serde::Serialize + for<'a> serde::Deserialize<'a>;
  type Error: snafu::Error + core::fmt::Debug + Send + Sync + 'static;

  fn caller<T>(&mut self, caller: T) -> Result<(), Self::Error>
  where
    T: Fn(crate::Call<<Self::Backend as Backend>::Intermediate>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>, <Self::Backend as Backend>::Error> + Send + Sync + 'static;

  fn receive(&self, call: crate::Call<<Self::Backend as Backend>::Intermediate>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>, Self::Error>;
}
