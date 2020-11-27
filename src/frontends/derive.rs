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
pub struct Derive<'a, B: Backend<'a>, C: Caller<'a, B>, R: Receiver<'a, B>> {
  #[allow(clippy::type_complexity)]
  _phantom: PhantomData<C>,

  receiver: R,
  caller: Option<Rc<dyn Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, B::Error> + 'a>>,
}

unsafe impl<'a, B: Backend<'a>, C: Caller<'a, B>, R: Receiver<'a, B>> Send for Derive<'a, B, C, R> {}

pub struct DeriveInit<'a, B: Backend<'a>, R: Receiver<'a, B>> {
  pub _phantom: PhantomData<&'a B>,

  pub receiver: R,
}

impl<'a, B: Backend<'a>, R: Receiver<'a, B>> DeriveInit<'a, B, R> {
  pub fn init<C: Caller<'a, B>>(self) -> Derive<'a, B, C, R> {
    Derive {
      _phantom: PhantomData,

      receiver: self.receiver,
      caller: None,
    }
  }
}

impl<'a, B, C: Caller<'a, B>, R: Receiver<'a, B>> interfaces::Frontend<'a, B> for Derive<'a, B, C, R>
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

pub trait Caller<'a, B: interfaces::Backend<'a>>: Clone {
  fn new<T>(caller: T) -> Self
  where
    T: Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, B::Error> + 'static + Send;

  #[allow(clippy::type_complexity)]
  fn get(&self) -> Rc<dyn Fn(&crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, B::Error> + 'a>;
}

pub trait Receiver<'a, B: interfaces::Backend<'a>> {
  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Error<B::Error>>;
}

// derive.register("add", |(a, b)| add(a, b)).unwrap();
pub struct Receive {
  pub offset: i32,
}

pub trait Frontend {
  fn add(a: i32, b: i32) -> i32;
  fn add_with_offset(&self, a: i32, b: i32) -> i32;
}

impl Frontend for Receive {
  fn add(a: i32, b: i32) -> i32 {
    a + b
  }

  fn add_with_offset(&self, a: i32, b: i32) -> i32 {
    a + b + self.offset
  }
}

impl<'a, B: interfaces::Backend<'a>> Receiver<'a, B> for Receive {
  fn receive(&self, call: &crate::Call<&B::Intermediate>) -> Result<crate::Reply<B::Intermediate>, Error<B::Error>> {
    match call.procedure.as_str() {
      "add" => {
        let payload = B::deserialize::<(i32, i32)>(call.payload)?;
        let reply = B::serialize(&Self::add(payload.0, payload.1))?;
        Ok(crate::Reply { payload: reply })
      }
      "add_with_offset" => {
        let payload = B::deserialize::<(i32, i32)>(call.payload)?;
        let reply = B::serialize(&self.add_with_offset(payload.0, payload.1))?;
        Ok(crate::Reply { payload: reply })
      }
      _ => Err(Error::UnknownProcedure {}),
    }
  }
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

impl<'a, B: interfaces::Backend<'a>> Caller<'a, B> for Call<'a, B> {
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
