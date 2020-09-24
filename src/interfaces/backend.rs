#[derive(Debug)]
pub enum Error {
}

pub struct Call<'a, T> {
  pub procedure: &'a str,
  pub payload: &'a T
}

pub struct Reply<T> {
  pub payload: T
}

pub trait Backend<'a> {
  type Intermediate;

  fn start(&mut self) -> Result<(), Error>;
  fn stop(&mut self) -> Result<(), Error>;

  fn receiver(&mut self, receiver: impl Fn(&Call<Self::Intermediate>) -> Reply<Self::Intermediate>) -> Result<(), Error>
    where Self::Intermediate: 'a;

  fn call(&mut self, call: &Call<Self::Intermediate>) -> Result<&Reply<Self::Intermediate>, Error>;
}
