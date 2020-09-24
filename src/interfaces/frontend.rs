
#[derive(Debug)]
pub struct Error {

}

pub trait Register {
  type Intermediate;

  fn call(&self, payload: & Self::Intermediate) -> Result<Self::Intermediate, Error>;
}
