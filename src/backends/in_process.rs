use crate::interfaces;
use crate::interfaces::backend::{Error, Result};

pub struct InProcess {
  #[allow(clippy::type_complexity)]
  receiver: Option<Box<dyn Fn(&crate::Call<&String>) -> Result<crate::Reply<String>> + Send>>,
}

impl Default for InProcess {
  fn default() -> Self {
    InProcess { receiver: None }
  }
}

impl InProcess {
  pub fn new() -> InProcess {
    InProcess::default()
  }
}

impl<'a> interfaces::Backend<'a> for InProcess {
  type Intermediate = String;

  fn start(&mut self) -> Result<()> {
    Ok(())
  }

  fn stop(&mut self) -> Result<()> {
    Ok(())
  }

  fn receiver<T>(&mut self, receiver: T) -> Result<()>
  where
    T: Fn(&crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>> + Send,
    T: 'static,
  {
    self.receiver = Some(Box::new(receiver));

    Ok(())
  }

  fn call(&mut self, call: &crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>> {
    self.receiver.as_ref().unwrap()(call)
  }

  fn serialize<T: serde::Serialize>(from: &T) -> Result<String> {
    serde_json::to_string(from).map_err(|e| interfaces::backend::Error::Serialize { serializer: e.to_string() })
  }

  fn deserialize<'b, T>(from: &'b Self::Intermediate) -> Result<T>
  where
    T: for<'de> serde::Deserialize<'de>,
  {
    serde_json::from_str(&from).map_err(|e| Error::Deserialize { deserializer: e.to_string() })
  }
}
