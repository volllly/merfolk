use crate::interfaces::backend;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
  Serialize { from: serde_json::Error },
  Deserialize { from: serde_json::Error },
}

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
}

impl<'a> backend::Backend<'a> for Empty {
  type Intermediate = String;
  type Error = Error;

  fn start(&mut self) -> Result<(), Self::Error> {
    self.started = true;
    Ok(())
  }

  fn stop(&mut self) -> Result<(), Self::Error> {
    self.started = false;
    Ok(())
  }

  #[allow(unused_variables)]
  fn receiver<T>(&mut self, receiver: T) -> Result<(), Self::Error>
  where
    T: Fn(&crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>, Self::Error>,
    T: 'a,
  {
    Ok(())
  }

  #[allow(unused_variables)]
  fn call(&mut self, call: &crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>, Self::Error> {
    Ok(crate::Reply { payload: call.payload.clone() })
  }

  fn serialize<T: serde::Serialize>(from: &T) -> Result<String, Self::Error> {
    serde_json::to_string(from).map_err(|e| Error::Deserialize { from: e })
  }

  fn deserialize<'b, T>(from: &'b Self::Intermediate) -> Result<T, Self::Error>
  where
    T: for<'de> serde::Deserialize<'de>,
  {
    serde_json::from_str(&from).map_err(|e| Error::Deserialize { from: e })
  }
}
