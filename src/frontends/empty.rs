use crate::interfaces::{backend, frontend};

#[derive(Debug)]
pub struct Empty {}

impl Default for Empty {
  fn default() -> Self {
    Empty {}
  }
}

impl Empty {
  pub fn new() -> Empty {
    Empty::default()
  }
}

impl<'a, B> frontend::Frontend<'a, B> for Empty
where
  B: backend::Backend<'a>,
{
  type Intermediate = String;

  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, frontend::Error> {
    let (a, b) = B::deserialize::<(i32, i32)>(call.payload).unwrap();
    let r = a + b;
    Ok(crate::Reply { payload: B::serialize(&r).unwrap() })
  }
}
