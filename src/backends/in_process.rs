use crate::interfaces;
use crate::interfaces::backend;

pub struct InProcess {
  #[allow(clippy::type_complexity)]
  receiver: Option<Box<dyn Fn(&crate::Call<&String>) -> Result<crate::Reply<String>, crate::Error> + Send>>,
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

  fn start(&mut self) -> Result<(), backend::Error> {
    Ok(())
  }

  fn stop(&mut self) -> Result<(), backend::Error> {
    Ok(())
  }

  fn receiver<T>(&mut self, receiver: T) -> Result<(), backend::Error>
  where
    T: Fn(&crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>, crate::Error> + Send,
    T: 'static,
  {
    self.receiver = Some(Box::new(receiver));

    Ok(())
  }

  fn call(&mut self, call: &crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>, backend::Error> {
    self.receiver.as_ref().unwrap()(call).map_err(|e| backend::Error::Deserialize(Some(format!("{:?}", e))))
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
