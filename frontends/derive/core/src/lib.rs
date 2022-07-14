pub use merfolk_frontend_derive_macros::frontend;
#[cfg(feature = "std")]
use thiserror::Error;

pub mod reexports {
  pub use anyhow;
  pub use derive_builder;
  pub use merfolk;
}

#[cfg(feature = "std")]
#[derive(Debug, Error)]
pub enum Error {
  #[error("backend error: {0}")]
  FromBackend(#[from] anyhow::Error),
  #[error("unknown procedure: {procedure}")]
  UnknownProcedure { procedure: String },
  #[error("error locking mutex")]
  MutexLock,
}
