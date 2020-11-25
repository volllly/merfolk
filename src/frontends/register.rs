use interfaces::Backend;

use crate::interfaces;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Register<'a, B: Backend<'a>> {
  #[allow(clippy::type_complexity)]
  procedures: Arc<Mutex<HashMap<String, Box<dyn Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, interfaces::frontend::Error> + 'a>>>>,
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

  pub fn register<P, C: for<'de> serde::Deserialize<'de>, R: serde::Serialize>(&self, name: &str, procedure: P) -> Result<(), interfaces::frontend::Error>
  where
    P: Fn(C) -> R + 'a,
  {
    self.procedures.lock().unwrap().insert(
      name.to_string(),
      Box::new(move |call: &crate::Call<&B::Intermediate>| {
        let reply = procedure(B::deserialize::<C>(call.payload).unwrap());
        Ok(crate::Reply {
          payload: B::serialize::<R>(&reply).unwrap(),
        })
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

  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, interfaces::frontend::Error> {
    self.procedures.lock().unwrap().get(&call.procedure).unwrap()(call)
  }
}
