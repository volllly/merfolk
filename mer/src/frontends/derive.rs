use interfaces::Backend;

use crate::interfaces;

use core::marker::PhantomData;

use alloc::rc::Rc;

use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error<B: core::fmt::Display> {
  FromBackend { from: B },
  UnknownProcedure {},
}

impl<B: snafu::Error> From<B> for Error<B> {
  fn from(from: B) -> Self {
    Error::FromBackend { from }
  }
}

#[allow(clippy::type_complexity)]
pub struct Derive<'a, B: Backend<'a>, C: Caller<'a>, R: Receiver<'a, B>> {
  #[allow(clippy::type_complexity)]
  _phantom: PhantomData<C>,

  receiver: R,
  caller: Option<Rc<dyn Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, B::Error> + 'a>>,
}

unsafe impl<'a, B: Backend<'a>, C: Caller<'a>, R: Receiver<'a, B>> Send for Derive<'a, B, C, R> {}

pub struct DeriveInit<'a, B: Backend<'a>, R: Receiver<'a, B>> {
  pub _phantom: PhantomData<&'a B>,

  pub receiver: R,
}

impl<'a, B: Backend<'a>, R: Receiver<'a, B>> DeriveInit<'a, B, R> {
  pub fn init<C: Caller<'a>>(self) -> Derive<'a, B, C, R> {
    Derive {
      _phantom: PhantomData,

      receiver: self.receiver,
      caller: None,
    }
  }
}

impl<'a, B, C: Caller<'a, B = B>, R: Receiver<'a, B>> interfaces::Frontend<'a, B> for Derive<'a, B, C, R>
where
  B: interfaces::Backend<'a>,
{
  type Intermediate = String;
  type Error = Error<B::Error>;
  type Call = C;

  fn caller<T>(&mut self, caller: T) -> Result<C, Self::Error>
  where
    T: Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, B::Error> + Send,
    T: 'static,
  {
    let call = C::new(caller);
    self.caller = Some(call.get());
    Ok(call)
  }

  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Error<B::Error>> {
    self.receiver.receive(call)
  }
}

pub trait Caller<'a>: Clone {
  type B: Backend<'a>;

  fn new<T>(caller: T) -> Self
  where
    T: Fn(&crate::Call<&<Self::B as interfaces::Backend<'a>>::Intermediate>) -> Result<crate::Reply<<Self::B as interfaces::Backend<'a>>::Intermediate>, <Self::B as interfaces::Backend<'a>>::Error> + 'static + Send;

  #[allow(clippy::type_complexity)]
  fn get(&self) -> Rc<dyn Fn(&crate::Call<&<Self::B as interfaces::Backend<'a>>::Intermediate>) -> Result<crate::Reply<<Self::B as interfaces::Backend<'a>>::Intermediate>, <Self::B as interfaces::Backend<'a>>::Error> + 'a>;
}

pub trait Receiver<'a, B: interfaces::Backend<'a>> {
  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Error<B::Error>>;
}

pub struct Call<'a, B: interfaces::Backend<'a>> {
  #[allow(clippy::type_complexity)]
  pub call: Rc<dyn Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, B::Error> + 'a>,
}

impl<'a, B: interfaces::Backend<'a>> Clone for Call<'a, B> {
  fn clone(&self) -> Self {
    Self { call: Rc::clone(&self.call) }
  }
}

impl<'a, B: interfaces::Backend<'a>> Caller<'a> for Call<'a, B> {
  type B = B;

  fn new<T>(caller: T) -> Self
  where
    T: Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, B::Error> + 'static + Send,
  {
    Self { call: Rc::new(caller) }
  }

  #[allow(clippy::type_complexity)]
  fn get(&self) -> Rc<dyn Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, B::Error> + 'a> {
    Rc::clone(&self.call)
  }
}

impl<'a, B: interfaces::Backend<'a>> Call<'a, B> {
  pub fn add(&self, a: i32, b: i32) -> Result<i32, B::Error> {
    Ok(B::deserialize(
      &(self.call)(&crate::Call {
        procedure: "add".to_string(),
        payload: &B::serialize(&(a, b)).unwrap(),
      })?
      .payload,
    )?)
  }

  pub fn add_with_offset(&self, a: i32, b: i32) -> Result<i32, B::Error> {
    Ok(B::deserialize(
      &(self.call)(&crate::Call {
        procedure: "add".to_string(),
        payload: &B::serialize(&(a, b)).unwrap(),
      })?
      .payload,
    )?)
  }
}
