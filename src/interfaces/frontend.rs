use crate::interfaces::backend;
use alloc::rc::Rc;

pub trait Frontend<'a, B>: Send
where
  B: backend::Backend<'a>,
{
  type Intermediate: serde::Serialize + serde::Deserialize<'a>;
  type Error: snafu::Error + core::fmt::Debug;
  type Call;

  fn caller<T>(&mut self, caller: T) -> Result<Rc<Self::Call>, Self::Error>
  where
    T: Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, B::Error> + 'a + Send,
    T: 'static;

  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Self::Error>;
}
