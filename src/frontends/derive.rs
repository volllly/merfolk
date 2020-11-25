use crate::interfaces;
use crate::interfaces::frontend::Result;

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

  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>> {
    let (a, b) = B::deserialize::<(i32, i32)>(call.payload).unwrap();
    let r = a + b;
    Ok(crate::Reply { payload: B::serialize(&r).unwrap() })
  }
}
