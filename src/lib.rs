#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(test)]
mod tests;

#[cfg(feature = "backends")]
pub mod backends;
#[cfg(feature = "frontends")]
pub mod frontends;
pub mod helpers;
pub mod interfaces;

use helpers::smart_pointer::*;

// #[derive(Debug, Snafu)]
// pub enum Error<B, F> where B: snafu::Error, F: snafu::Error {
//   MakeCall { source: B },
//   ReceiveCall { source: F },
//   Start { source: B },
//   Stop { source: B },
// }

pub struct Call<T> {
  pub procedure: String,
  pub payload: T,
}
unsafe impl<T> Send for Call<T> where T: Send {}

pub struct Reply<T> {
  pub payload: T,
}
unsafe impl<T> Send for Reply<T> where T: Send {}

pub struct Caller<'a, B: interfaces::Backend<'a>> {
  #[allow(clippy::type_complexity)]
  call: Box<dyn Fn(&Call<&B::Intermediate>) -> Result<Reply<B::Intermediate>, B::Error> + 'a>,
}

pub trait AutomaticCall<'a, B: interfaces::Backend<'a>> {
  #[allow(clippy::type_complexity)]
  fn call<R>(&self, call: &Call<&B::Intermediate>) -> Result<Reply<R>, B::Error>
  where
    R: for<'de> serde::Deserialize<'de>;
}

impl<'a, B: interfaces::Backend<'a>> AutomaticCall<'a, B> for Caller<'a, B> {
  #[allow(clippy::type_complexity)]
  fn call<R>(&self, call: &Call<&B::Intermediate>) -> Result<Reply<R>, B::Error>
  where
    R: for<'de> serde::Deserialize<'de>,
  {
    let reply = (self.call)(call)?;
    Ok(Reply {
      payload: B::deserialize(&reply.payload)?,
    })
  }
}

// pub trait ManualCall<'a, B> {
//   fn manual<C>(&self, procedure: &str, payload: Box<dyn erased_serde::Serialize>) -> Result<B, backend::Error>;
// }

// impl<'a, B: backend::Backend<'a>, F: frontend::Frontend<'a>> ManualCall<'a, B> for Caller<'a, B, F> {
//   fn manual<C>(&self, procedure: &str, payload: Box<dyn erased_serde::Serialize>) -> Result<B, backend::Error> {
//     (self.call)(&Call { procedure, payload }).map(|r| r.payload)
//   }
// }

pub struct Mer<'a, B, F>
where
  B: interfaces::Backend<'a>,
  B: 'a,
  F: interfaces::Frontend<'a, B>,
  F: 'a,
{
  backend: SmartPointer<B>,
  frontend: SmartPointer<F>,

  #[allow(dead_code)]
  call: Caller<'a, B>,
}

pub struct MerInit<B, F> {
  pub backend: B,
  pub frontend: F,
}

impl<'a, B, F> MerInit<B, F>
where
  B: interfaces::Backend<'a>,
  B: 'a,
  F: interfaces::Frontend<'a, B>,
  F: 'static,
{
  pub fn init(self) -> Mer<'a, B, F> {
    let backend = smart_pointer!(self.backend);
    let frontend = smart_pointer!(self.frontend);

    let frontend_receiver = clone!(frontend);
    access_mut!(backend)
      .unwrap()
      .receiver(move |call: &Call<&B::Intermediate>| Ok(access!(frontend_receiver).unwrap().receive(call).unwrap()))
      .unwrap();

    Mer {
      backend: clone!(backend),
      frontend,

      call: Caller {
        call: Box::new(move |call: &Call<&B::Intermediate>| access_mut!(backend).unwrap().call(call)),
      },
    }
  }
}

impl<'a, B: interfaces::Backend<'a>, F: interfaces::Frontend<'a, B>> Mer<'a, B, F> {
  pub fn start(&mut self) -> Result<(), B::Error> {
    access_mut!(self.backend).unwrap().start()
  }

  pub fn stop(&mut self) -> Result<(), B::Error> {
    access_mut!(self.backend).unwrap().stop()
  }

  pub fn receive(&self, call: &Call<&B::Intermediate>) -> Result<Reply<B::Intermediate>, F::Error> {
    access_mut!(self.frontend).unwrap().receive(call)
  }
}
