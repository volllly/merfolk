use super::Backend;

use anyhow::Result;

/// [`Middleware`]s are used to Modify RPCs or to add extra functionality to the framework.
///
/// # Outgoing RPCs
/// The outgoing [`Call`](crate::Call)<[`Intermediate`](Backend::Intermediate)`>`s are processed by the [`wrap_call`](Middleware::wrap_call) function and passed on.
/// The incomming [`Reply`](crate::Reply)<[`Intermediate`](Backend::Intermediate)`>`s are processed by the [`unwrap_reply`](Middleware::unwrap_reply) function and passed on.
///
/// # Incomming RPCs
/// The incomming [`Call`](crate::Call)<[`Intermediate`](Backend::Intermediate)`>`s are processed by the [`unwrap_call`](Middleware::unwrap_call) function and passed on.
/// The outgoing [`Reply`](crate::Reply)<[`Intermediate`](Backend::Intermediate)`>`s are processed by the [`wrap_reply`](Middleware::wrap_reply) function and passed on.
///
/// # Examples
/// For examples look at the provided [Middlewares](Middleware):
/// * [`Authentication`](mer_middleware_authentication)
/// * [`Router`](mer_middleware_router)

pub trait Middleware: Send {
  type Backend: Backend;

  /// Wraps the outgoing call [`Call`](crate::Call)<[`Intermediate`](Backend::Intermediate)`>`
  fn wrap_call(&self, call: Result<crate::Call<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Call<<Self::Backend as Backend>::Intermediate>>;

  /// Wraps the outgoing reply [`Reply`](crate::Reply)<[`Intermediate`](Backend::Intermediate)`>`
  fn wrap_reply(&self, call: Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>;

  /// Unwraps the incomming call [`Call`](crate::Call)<[`Intermediate`](Backend::Intermediate)`>`
  fn unwrap_call(&self, reply: Result<crate::Call<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Call<<Self::Backend as Backend>::Intermediate>>;

  /// Unwraps the imcomming reply [`Reply`](crate::Reply)<[`Intermediate`](Backend::Intermediate)`>`
  fn unwrap_reply(&self, reply: Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>;

  fn as_any(&mut self) -> &mut dyn core::any::Any;
}
