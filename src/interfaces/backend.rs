use serde::Deserialize;

#[derive(Debug)]
pub enum Error {
  Decode(String)
}

pub trait Backend {
  type Encoded;

  fn encode<T>(&self, tx: Tx<T>) -> Result<Self::Encoded, Error>;

  fn decode<'de, T: Deserialize<'de>>(&self, rx: &'de Self::Encoded) -> Result<Rx<T>, Error>;
}

pub struct Tx<'a, T> {
  pub procedure: &'a str,
  pub payload: T
}

pub struct Rx<'a, T> {
  pub procedure: &'a str,
  pub payload: T
}


