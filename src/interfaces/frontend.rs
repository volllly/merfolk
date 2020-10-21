#[derive(Debug)]
pub struct Error {}

pub trait Frontend<'a> {
  type Intermediate: serde::Serialize + serde::Deserialize<'a>;

  fn receive(
    &self,
    call: &crate::Call<&Self::Intermediate>,
  ) -> Result<crate::Reply<Box<dyn erased_serde::Serialize>>, Error>;

  fn serialize(&self, from: &dyn erased_serde::Serialize) -> &Self::Intermediate;
}

pub trait Procedure<'a> {
  type Intermediate: serde::Serialize + serde::Deserialize<'a>;

  fn call(&self, payload: &Self::Intermediate) -> Result<Self::Intermediate, Error>;
}
