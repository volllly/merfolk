use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
  Serialize { serializer: String },
  Deserialize { deserializer: String },
  SpeakingDisabled,
  Listen,
  Call { inner: String },
  Other,
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub trait Backend<'a>: Send {
  type Intermediate: serde::Serialize + serde::Deserialize<'a>;

  fn start(&mut self) -> Result<()>;
  fn stop(&mut self) -> Result<()>;

  fn receiver<T>(&mut self, receiver: T) -> Result<()>
  where
    T: Fn(&crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>> + Send,
    T: 'static;

  fn call(&mut self, call: &crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>>;

  fn serialize<T: serde::Serialize>(from: &T) -> Result<Self::Intermediate>;

  fn deserialize<'b, T>(from: &'b Self::Intermediate) -> Result<T>
  where
    T: for<'de> serde::Deserialize<'de>;
}
