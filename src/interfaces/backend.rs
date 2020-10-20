#[derive(Debug)]
pub enum Error {}

// EXPRTIMENTAL: pub trait Receiver<T> = Fn(&Call<T>) -> Reply<T>;

pub trait Backend<'a> {
  type Intermediate: serde::Serialize + serde::Deserialize<'a>;

  fn start(&mut self) -> Result<(), Error>;
  fn stop(&mut self) -> Result<(), Error>;

  fn receiver<T>(&mut self, receiver: T) -> Result<(), Error>
  where
    T: Fn(&crate::Call<Self::Intermediate>) -> crate::Reply<Self::Intermediate>,
    T: 'a,
    Self::Intermediate: 'a;

  fn call(&mut self, call: &crate::Call<Self::Intermediate>) -> Result<&crate::Reply<Self::Intermediate>, Error>;
}
