#[derive(Debug)]
pub enum Error {
  Serialize(Option<String>),
  Deserialize(Option<String>),
  Speak(Option<String>),
  Call(Option<String>),
  Other
}

// EXPRTIMENTAL: pub trait Receiver<T> = Fn(&Call<T>) -> Reply<T>;

pub trait Backend<'a> {
  type Intermediate: serde::Serialize + serde::Deserialize<'a>;

  fn start(&mut self) -> Result<(), Error>;
  fn stop(&mut self) -> Result<(), Error>;

  // fn receiver<T>(&mut self, receiver: T) -> Result<(), Error>
  // where
  //   T: Fn(&crate::Call<&dyn erased_serde::Serialize>) -> Result<crate::Reply<Self::Intermediate>, crate::Error>,
  //   T: 'a,
  //   Self::Intermediate: 'a;

  fn call(&mut self, call: &crate::Call<Box<dyn erased_serde::Serialize>>) -> Result<crate::Reply<Self::Intermediate>, Error>;

  fn serialize(&self, from: &dyn erased_serde::Serialize) -> Result<Self::Intermediate, Error>;

  fn deserialize<'b, T>(&self, from: &'b Self::Intermediate) -> Result<T, Error> where T: for<'de> serde::Deserialize<'de>;
}
