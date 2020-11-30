pub trait Backend<'a>: Send {
  type Intermediate: serde::Serialize + serde::Deserialize<'a>;
  type Error: snafu::Error + core::fmt::Debug;

  fn start(&mut self) -> Result<(), Self::Error>;
  fn stop(&mut self) -> Result<(), Self::Error>;

  fn receiver<T>(&mut self, receiver: T) -> Result<(), Self::Error>
  where
    T: Fn(&crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>, Self::Error> + Send,
    T: 'static;

  fn call(&mut self, call: &crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>, Self::Error>;

  fn serialize<T: serde::Serialize>(from: &T) -> Result<Self::Intermediate, Self::Error>;

  fn deserialize<'b, T>(from: &'b Self::Intermediate) -> Result<T, Self::Error>
  where
    T: for<'de> serde::Deserialize<'de>;
}
