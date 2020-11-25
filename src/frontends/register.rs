use interfaces::Backend;

use crate::interfaces;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

pub struct Register<'a, B: Backend<'a>> {
  #[allow(clippy::type_complexity)]
  procedures: Arc<Mutex<HashMap<String, Box<dyn Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Error<B::Error>> + 'a>>>>,
}

unsafe impl<'a, T: Backend<'a>> Send for Register<'a, T> {}

impl<'a, T: interfaces::Backend<'a>> Default for Register<'a, T> {
  fn default() -> Self {
    Register {
      procedures: Arc::new(Mutex::new(HashMap::new())),
    }
  }
}

impl<'a, B: Backend<'a>> Register<'a, B> {
  pub fn new() -> Register<'a, B> {
    Register::default()
  }

  pub fn register<P, C: for<'de> serde::Deserialize<'de>, R: serde::Serialize>(&self, name: &str, procedure: P) -> Result<(), Error<B::Error>>
  where
    P: Fn(C) -> R + 'a,
  {
    self.procedures.lock().unwrap().insert(
      name.to_string(),
      Box::new(move |call: &crate::Call<&B::Intermediate>| {
        let reply = procedure(B::deserialize::<C>(call.payload)?);
        Ok(crate::Reply { payload: B::serialize::<R>(&reply)? })
      }),
    );
    Ok(())
  }
}

impl<'a, B> interfaces::Frontend<'a, B> for Register<'a, B>
where
  B: interfaces::Backend<'a>,
{
  type Intermediate = String;
  type Error = Error<B::Error>;

  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Error<B::Error>> {
    self.procedures.lock().unwrap().get(&call.procedure).unwrap()(call)
  }
}
