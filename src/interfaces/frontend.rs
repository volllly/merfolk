use crate::interfaces::backend;

#[derive(Debug)]
pub enum Error {
  Serialize(Option<String>),
  Deserialize(Option<String>),
}

pub trait Frontend<'a, B>: Send
where
  B: backend::Backend<'a>,
{
  type Intermediate: serde::Serialize + serde::Deserialize<'a>;

  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Error>;
}
