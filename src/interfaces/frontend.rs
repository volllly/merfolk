#[derive(Debug)]
pub enum Error {
  Serialize(Option<String>),
  Deserialize(Option<String>),
}

pub trait Frontend<'a> {
  type Intermediate: serde::Serialize + serde::Deserialize<'a>;

  fn receive(&self, call: &crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Box<dyn erased_serde::Serialize>>, Error>;

  fn serialize(&self, from: &dyn erased_serde::Serialize) -> Result<Self::Intermediate, Error>;
  fn deserialize<T: serde::Deserialize<'a>>(&self, from: &'a Self::Intermediate) -> Result<T, Error>;
}
