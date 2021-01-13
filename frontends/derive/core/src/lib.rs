use snafu::Snafu;

pub use mer_frontend_derive_macros::frontend;

#[derive(Debug, Snafu)]
pub enum Error<B: core::fmt::Display> {
  FromBackend { from: B },
  UnknownProcedure,
  MutexLock,
}

impl<B: snafu::Error> From<B> for Error<B> {
  fn from(from: B) -> Self {
    Error::FromBackend { from }
  }
}
