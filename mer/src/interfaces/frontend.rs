use crate::interfaces::backend;

pub trait Frontend<'a, B>: Send
where
  B: backend::Backend<'a>,
{
  type Intermediate: serde::Serialize + serde::Deserialize<'a>;
  type Error: snafu::Error + core::fmt::Debug;

  fn caller<T>(&mut self, caller: smart_pointer_type!(T)) -> Result<(), Self::Error>
  where
    T: Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, B::Error> + 'a + Send;

  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Self::Error>;
}
