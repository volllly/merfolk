use crate::interfaces::{backend, frontend};

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
pub struct Empty {}

pub struct EmptyInit {}

impl Default for EmptyInit {
  fn default() -> Self {
    EmptyInit {}
  }
}

impl EmptyInit {
  pub fn init(self) -> Empty {
    Empty {}
  }
}

impl<'a, B> frontend::Frontend<'a, B> for Empty
where
  B: backend::Backend<'a>,
{
  type Intermediate = String;
  type Error = Error<B::Error>;
  type Call = ();

  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Self::Error> {
    let (a, b) = B::deserialize::<(i32, i32)>(call.payload)?;
    let r = a + b;
    Ok(crate::Reply { payload: B::serialize(&r)? })
  }

  #[allow(unused_variables)]
  fn caller<T>(&mut self, caller: T) -> Result<alloc::rc::Rc<Self::Call>, Self::Error>
  where
    T: Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, B::Error> + 'a + Send,
    T: 'static,
  {
    Ok(alloc::rc::Rc::new(()))
  }
}
