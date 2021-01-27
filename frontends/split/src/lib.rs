use std::marker::PhantomData;

use mer::{
  interfaces::{Backend, Frontend},
  Call, Reply,
};

use anyhow::Result;
use thiserror::Error;

use log::trace;

#[derive(Debug, Error)]
pub enum Error {
  #[error("backend error: {0}")]
  FromBackend(#[from] anyhow::Error),
}

pub struct Split<'a, B, FC, FR>
where
  B: Backend + 'a,
  FC: Frontend<Backend = B> + 'a,
  FR: Frontend<Backend = B> + 'a,
{
  __phantom: PhantomData<&'a B>,
  pub caller: FC,
  pub receiver: FR,
}

unsafe impl<'a, B, FC, FR> Send for Split<'a, B, FC, FR>
where
  B: Backend,
  FC: Frontend<Backend = B>,
  FR: Frontend<Backend = B>,
{
}

pub struct SplitInit<B, FC, FR>
where
  B: Backend,
  FC: Frontend<Backend = B>,
  FR: Frontend<Backend = B>,
{
  pub caller: FC,
  pub receiver: FR,
}

impl<'a, B, FC, FR> SplitInit<B, FC, FR>
where
  B: Backend,
  FC: Frontend<Backend = B>,
  FR: Frontend<Backend = B>,
{
  pub fn init(self) -> Split<'a, B, FC, FR> {
    trace!("initialize split");

    Split {
      __phantom: PhantomData,

      caller: self.caller,
      receiver: self.receiver,
    }
  }
}

impl<'a, B, FC, FR> Frontend for Split<'a, B, FC, FR>
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
    trace!("split caller");

    self.caller.register(caller)
  }

  #[allow(clippy::type_complexity)]
  fn receive(&self, call: Call<<Self::Backend as Backend>::Intermediate>) -> Result<Reply<<Self::Backend as Backend>::Intermediate>> {
    trace!("receive call");

    self.receiver.receive(call)
  }
}
