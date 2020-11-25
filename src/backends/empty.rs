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

  #[allow(unused_variables)]
  fn receiver<T>(&mut self, receiver: T) -> Result<(), backend::Error>
  where
    T: Fn(&crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>, crate::Error>,
    T: 'a,
  {
    Ok(())
  }

  #[allow(unused_variables)]
  fn call(&mut self, call: &crate::Call<Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>, backend::Error> {
    //let ser = Self::serialize(&call.payload).unwrap();
    println!("{}: {}", call.procedure, call.payload);
    Ok(crate::Reply { payload: call.payload.clone() })
  }

  fn serialize<T: serde::Serialize>(from: &T) -> Result<String, backend::Error> {
    serde_json::to_string(from).map_err(|e| backend::Error::Serialize(Some(e.to_string())))
  }

  fn deserialize<'b, T>(from: &'b Self::Intermediate) -> Result<T, backend::Error>
  where
    T: for<'de> serde::Deserialize<'de>,
  {
    serde_json::from_str(&from).map_err(|e| backend::Error::Deserialize(Some(e.to_string() + "; from: " + from)))
  }
}
