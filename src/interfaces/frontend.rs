use crate::interfaces::backend;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
  Serialize,
  Deserialize,
}
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub trait Frontend<'a, B>: Send
where
  B: backend::Backend<'a>,
{
  type Intermediate: serde::Serialize + serde::Deserialize<'a>;

  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>>;
}
