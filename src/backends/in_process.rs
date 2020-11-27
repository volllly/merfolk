use crate::interfaces;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
  Serialize { from: serde_json::Error },
  Deserialize { from: serde_json::Error },
}

pub struct InProcess {
  #[allow(clippy::type_complexity)]
  receiver: Option<Box<dyn Fn(&crate::Call<&String>) -> Result<crate::Reply<String>, Error> + Send>>,
}

pub struct InProcessInit {}

impl Default for InProcessInit {
  fn default() -> Self {
    InProcessInit {}
  }
}

impl InProcessInit {
  pub fn init(self) -> InProcess {
    InProcess { receiver: None }
  }
}

impl<'a> interfaces::Backend<'a> for InProcess {
  type Intermediate = String;
  type Error = Error;

  fn start(&mut self) -> Result<(), Self::Error> {
    Ok(())
  }

  fn stop(&mut self) -> Result<(), Self::Error> {
    Ok(())
  }

  fn receiver<T>(&mut self, receiver: T) -> Result<(), Self::Error>
  where
    T: Fn(&crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>, Self::Error> + Send,
    T: 'static,
  {
    self.receiver = Some(Box::new(receiver));

    Ok(())
  }

  fn call(&mut self, call: &crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>, Self::Error> {
    self.receiver.as_ref().unwrap()(call)
  }

  fn serialize<T: serde::Serialize>(from: &T) -> Result<String, Self::Error> {
    serde_json::to_string(from).map_err(|e| Error::Serialize { from: e })
  }

  fn deserialize<'b, T>(from: &'b Self::Intermediate) -> Result<T, Self::Error>
  where
    T: for<'de> serde::Deserialize<'de>,
  {
    serde_json::from_str(&from).map_err(|e| Error::Deserialize { from: e })
  }
}
