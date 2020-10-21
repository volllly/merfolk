use crate::interfaces::frontend;

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

impl<'a> frontend::Frontend<'a> for Empty {
  type Intermediate = String;

  fn receive(&self, call: &crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Box<dyn erased_serde::Serialize>>, frontend::Error> {
    let ser = self.serialize(call.payload).unwrap();
    println!("{}: {}", call.procedure, ser);
    Ok(crate::Reply { payload: Box::new(ser) })
  }

  fn serialize(&self, from: &dyn erased_serde::Serialize) -> Result<String, frontend::Error> {
    serde_json::to_string(from).map_err(|e| frontend::Error::Serialize(Some(e.to_string())))
  }

  fn deserialize<T: serde::Deserialize<'a>>(&self, from: &'a Self::Intermediate) -> Result<T, frontend::Error> {
    serde_json::from_str(&from).map_err(|e| frontend::Error::Deserialize(Some(e.to_string())))
  }
}
