#![cfg_attr(not(feature = "std"), no_std)]

use std::marker::PhantomData;

use anyhow::Result;
use log::trace;
use merfolk::{
  interfaces::{Backend, Frontend},
  Call, Reply,
};

#[derive(derive_builder::Builder)]
#[builder(pattern = "owned")]
#[cfg_attr(not(feature = "std"), builder(no_std))]
pub struct Duplex<'a, B, FC, FR>
where
  B: Backend + 'a,
  FC: Frontend<Backend = B> + 'a,
  FR: Frontend<Backend = B> + 'a,
{
  #[builder(private, default = "PhantomData")]
  __phantom: PhantomData<&'a B>,

  pub caller: FC,

  pub receiver: FR,
}

impl<'a, B, FC, FR> Duplex<'a, B, FC, FR>
where
  B: Backend + 'a,
  FC: Frontend<Backend = B> + 'a,
  FR: Frontend<Backend = B> + 'a,
{
  pub fn builder() -> DuplexBuilder<'a, B, FC, FR> {
    DuplexBuilder::default()
  }
}

unsafe impl<'a, B, FC, FR> Send for Duplex<'a, B, FC, FR>
where
  B: Backend,
  FC: Frontend<Backend = B>,
  FR: Frontend<Backend = B>,
{
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
