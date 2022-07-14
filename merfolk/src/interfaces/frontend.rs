use anyhow::Result;

use super::Backend;

/// The [`Frontend`] is responsible for processing incomming RPCs and for passing on outgoing RPCs to the [`Backend`].
/// The [`Frontend`] provides a way to make calls on the client side and a way to register procedures on the server side.
///
/// # Incomming RPCs
/// The [`Frontend`] is acting as server.
///
/// Incomming [`Call`](crate::Call)<[`Intermediate`](Backend::Intermediate)`>`s are received from the [`Backend`].
/// The calls are deserialized via [`Backend::deserialize`](crate::interfaces::Backend::deserialize) to the correct type.
/// The calls are processed and the replys serialized via [`Backend::serialize`](crate::interfaces::Backend::serialize) and passed on to the [`Backend`] as [`Reply`](crate::Reply)<[`Intermediate`](Backend::Intermediate)`>`.
///
/// # Outgoing RPCs
/// The [`Frontend`] is acting as client.
///
/// The [`Frontend`] serializes the calls via [`Backend::serialize`](crate::interfaces::Backend::serialize)
/// and passes them to the [`Backend`] as [`Call`](crate::Call)<[`Intermediate`](Backend::Intermediate)`>`.
/// The replies are received from the [`Backend`] deserialized via [`Backend::deserialize`](crate::interfaces::Backend::deserialize) passed on as response.
///
/// # Examples
/// For examples look at the provided [`Frontend`]s:
/// * [`Derive`](/merfolk_frontend_derive)
/// * [`Logger`](/merfolk_frontend_logger)
/// * [`Register`](/merfolk_frontend_register)

pub trait Frontend: Send {
  /// The used  [`Backend`].
  type Backend: Backend;

  /// Registers the client callback function from [`Mer`](crate::Mer). The callback is used to pass outgoing [`Call`](crate::Call)s to the [`Backend`].
  fn register<T>(&mut self, caller: T) -> Result<()>
  where
    T: Fn(crate::Call<<Self::Backend as Backend>::Intermediate>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>> + Send + Sync + 'static;

  /// This function is called by the [`Backend`] for incomming [`Call`](crate::Call)s.
  fn receive(&self, call: crate::Call<<Self::Backend as Backend>::Intermediate>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>;
}
