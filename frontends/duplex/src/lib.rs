#![cfg_attr(not(feature = "std"), no_std)]

use std::marker::PhantomData;

use mer::{
  interfaces::{Backend, Frontend},
  Call, Reply,
};

use anyhow::Result;

use log::trace;

pub struct Duplex<'a, B, FC, FR>
where
  B: Backend + 'a,
  FC: Frontend<Backend = B> + 'a,
  FR: Frontend<Backend = B> + 'a,
{
  __phantom: PhantomData<&'a B>,
  pub caller: FC,
  pub receiver: FR,
}

unsafe impl<'a, B, FC, FR> Send for Duplex<'a, B, FC, FR>
where
  B: Backend,
  FC: Frontend<Backend = B>,
  FR: Frontend<Backend = B>,
{
}

pub struct DuplexInit<B, FC, FR>
where
  B: Backend,
  FC: Frontend<Backend = B>,
  FR: Frontend<Backend = B>,
{
  pub caller: FC,
  pub receiver: FR,
}

impl<'a, B, FC, FR> DuplexInit<B, FC, FR>
where
  B: Backend,
  FC: Frontend<Backend = B>,
  FR: Frontend<Backend = B>,
{
  pub fn init(self) -> Duplex<'a, B, FC, FR> {
    trace!("initialize duplex");

    Duplex {
      __phantom: PhantomData,

      caller: self.caller,
      receiver: self.receiver,
    }
  }
}

impl<'a, B, FC, FR> Frontend for Duplex<'a, B, FC, FR>
where
  B: Backend,
  FC: Frontend<Backend = B>,
  FR: Frontend<Backend = B>,
{
  type Backend = B;

  fn register<T>(&mut self, caller: T) -> Result<()>
  where
    T: Fn(Call<<Self::Backend as Backend>::Intermediate>) -> Result<Reply<<Self::Backend as Backend>::Intermediate>> + 'static + Send + Sync,
  {
    trace!("duplex caller");

    self.caller.register(caller)
  }

  #[allow(clippy::type_complexity)]
  fn receive(&self, call: Call<<Self::Backend as Backend>::Intermediate>) -> Result<Reply<<Self::Backend as Backend>::Intermediate>> {
    trace!("receive call");

    self.receiver.receive(call)
  }
}
