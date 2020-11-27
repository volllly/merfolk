use interfaces::Backend;

use crate::interfaces;

use alloc::rc::Rc;
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
  caller: Option<Rc<Call<'a, B>>>,
}

unsafe impl<'a, T: Backend<'a>> Send for Register<'a, T> {}

pub struct RegisterInit {}

impl Default for RegisterInit {
  fn default() -> Self {
    RegisterInit {}
  }
}

impl RegisterInit {
  pub fn init<'a, B: Backend<'a>>(self) -> Register<'a, B> {
    Register {
      procedures: Arc::new(Mutex::new(HashMap::new())),
      caller: None,
    }
  }
}

impl<'a, B: Backend<'a>> Register<'a, B> {
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

pub struct Call<'a, B: interfaces::Backend<'a>> {
  #[allow(clippy::type_complexity)]
  pub call: Box<dyn Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, B::Error> + 'a>,
}

impl<'a, B> interfaces::Frontend<'a, B> for Register<'a, B>
where
  B: interfaces::Backend<'a>,
{
  type Intermediate = String;
  type Error = Error<B::Error>;
  type Call = Call<'a, B>;

  fn caller<T>(&mut self, caller: T) -> Result<Rc<Self::Call>, Self::Error>
  where
    T: Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, B::Error> + 'a + Send,
    T: 'static,
  {
    let call = Rc::new(Call { call: Box::new(caller) });
    self.caller = Some(Rc::clone(&call));
    Ok(call)
  }

  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Error<B::Error>> {
    self.procedures.lock().unwrap().get(&call.procedure).unwrap()(call)
  }
}
