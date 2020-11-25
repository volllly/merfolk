use crate::interfaces;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error<B: core::fmt::Display> {
  FromBackend { from: B },
}

impl<B: snafu::Error> From<B> for Error<B> {
  fn from(from: B) -> Self {
    Error::FromBackend { from }
  }
}

#[derive(Debug)]
pub struct Derive {}

impl Default for Derive {
  fn default() -> Self {
    Derive {}
  }
}

impl Derive {
  pub fn new() -> Derive {
    Derive::default()
  }
}

impl<'a, B> interfaces::Frontend<'a, B> for Derive
where
  B: interfaces::Backend<'a>,
{
  type Intermediate = String;
  type Error = Error<B::Error>;

  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Self::Error> {
    let (a, b) = B::deserialize::<(i32, i32)>(call.payload)?;
    let r = a + b;
    Ok(crate::Reply { payload: B::serialize(&r)? })
  }
}
