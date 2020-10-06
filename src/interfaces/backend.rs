#[derive(Debug)]
pub enum Error {}

pub struct Call<'a, T> {
  pub procedure: &'a str,
  pub payload: &'a T,
}

pub struct Reply<T> {
  pub payload: T,
}

// Exprtimental
// pub trait Receiver<T> = Fn(&Call<T>) -> Reply<T>;

pub trait Backend<'a> {
  type Intermediate: serde::Serialize + serde::Deserialize<'a>;

  fn start(&mut self) -> Result<(), Error>;
  fn stop(&mut self) -> Result<(), Error>;

  fn receiver<T>(&mut self, receiver: T) -> Result<(), Error>
  where
    T: Fn(&Call<Self::Intermediate>) -> Reply<Self::Intermediate>,
    T: 'a,
    Self::Intermediate: 'a;

  fn call(&mut self, call: &Call<Self::Intermediate>) -> Result<&Reply<Self::Intermediate>, Error>;
}
