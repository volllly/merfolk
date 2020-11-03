use crate::interfaces::backend;

pub struct Empty {
  started: bool,
}

impl Default for Empty {
  fn default() -> Self {
    Empty { started: false }
  }
}

impl Empty {
  pub fn new() -> Empty {
    Empty::default()
  }

  pub fn trigger(self) {
    // self.receiver.unwrap()(&crate::Call {
    //   procedure: &"",
    //   payload: (),
    // });
  }
}

impl<'a> backend::Backend<'a> for Empty {
  type Intermediate = String;

  fn start(&mut self) -> Result<(), backend::Error> {
    self.started = true;
    Ok(())
  }

  fn stop(&mut self) -> Result<(), backend::Error> {
    self.started = false;
    Ok(())
  }

  // fn receiver<T>(&mut self, receiver: T) -> Result<(), backend::Error>
  // where
  //   T: Fn(&crate::Call<Self::Intermediate>) -> crate::Reply<Self::Intermediate>,
  //   T: 'a,
  //   Self::Intermediate: 'a,
  // {
  //   self.receiver = Some(Box::new(receiver));
  //   Ok(())
  // }

  #[allow(unused_variables)]
  fn call(&mut self, call: &crate::Call<Box<dyn erased_serde::Serialize>>) -> Result<crate::Reply<Self::Intermediate>, backend::Error> {
    let ser = self.serialize(&call.payload).unwrap();
    println!("{}: {}", call.procedure, ser);
    Ok(crate::Reply { payload: ser })
  }

  fn serialize(&self, from: &dyn erased_serde::Serialize) -> Result<String, backend::Error> {
    serde_json::to_string(from).map_err(|e| backend::Error::Serialize(Some(e.to_string())))
  }

  fn deserialize<'b, T>(&self, from: &'b Self::Intermediate) -> Result<T, backend::Error> where T: for<'de> serde::Deserialize<'de> {
    serde_json::from_str(&from).map_err(|e| backend::Error::Deserialize(Some(e.to_string())))
  }
}
