use crate::interfaces::backend;

#[derive(Debug)]
pub enum Error {
  Serialize(Option<String>),
  Deserialize(Option<String>),
}

pub trait Frontend<'a, B> : Send where B: backend::Backend<'a> {
  type Intermediate: serde::Serialize + serde::Deserialize<'a>;

  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Error>;

  //fn serialize(&self, from: &B::Intermediate) -> Result<Self::Intermediate, Error>;
  //fn deserialize<D: serde::Deserialize<'a>>(&self, from: &'a Self::Intermediate) -> Result<D, Error>;
}
