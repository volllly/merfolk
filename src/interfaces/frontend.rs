#[derive(Debug)]
pub struct Error {}

pub trait Procedure<'a> {
  type Intermediate: serde::Serialize + serde::Deserialize<'a>;

  fn call(&self, payload: &Self::Intermediate) -> Result<Self::Intermediate, Error>;
}
