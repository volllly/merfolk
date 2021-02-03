use anyhow::Result;

/// The [`Backend`] is responsible for sending and receiving RPCs.
/// The [`Backend`] serializes and deserializes the packages and sends and reveices them over a chsen channel.
///
/// # Incomming
/// The [`Backend`] is acting as server.
///
/// Incomming calls from the client are passed on to the [`Frontend`](crate::interfaces::Frontend) as [`Call`](crate::Call)<[`Intermediate`](Backend::Intermediate)`>`.
/// The [`Reply`](crate::Reply)<[`Intermediate`](Backend::Intermediate)`>`s from the [`Frontend`](crate::interfaces::Frontend) are sent back to the client.
///
/// # Outgoing
/// The [`Backend`] is acting as client.
///
/// The [`Frontend`](crate::interfaces::Frontend) passes [`Call`](crate::Call)<[`Intermediate`](Backend::Intermediate)`>`s to the [`Backend`] which sends them to the server.
/// The replies are received from the server and passed on to the [`Frontend`](crate::interfaces::Frontend) as [`Reply`](crate::Reply)<[`Intermediate`](Backend::Intermediate)`>`.
///
/// # Examples
/// For examples look at the provided [`Backend`]s:
/// * [`Http`](mer_backend_http::Http)
/// * [`InProcess`](mer_backend_in_process::InProcess)
/// * [`SerialPort`](mer_backend_serialport::SerialPort)

pub trait Backend: Send {
  /// The Intermediate type required by the [`Backend`].
  type Intermediate: serde::Serialize + for<'a> serde::Deserialize<'a>;

  /// Starts the [`Backend`]. Can be optional.
  fn start(&mut self) -> Result<()>;
  /// Stops the [`Backend`]. Can be optional.
  fn stop(&mut self) -> Result<()>;

  /// Registers the server callback function from [`Mer`](crate::Mer). The callback is used to pass incomming [`Call`](crate::Call)s to the [`Frontend`](crate::interfaces::Frontend).
  fn register<T>(&mut self, receiver: T) -> Result<()>
  where
    T: Fn(crate::Call<<Self as Backend>::Intermediate>) -> Result<crate::Reply<<Self as Backend>::Intermediate>> + Send + Sync + 'static;

  /// This function is called by the [`Frontend`](crate::interfaces::Frontend) for outgoing [`Call`](crate::Call)s.
  fn call(&mut self, call: crate::Call<<Self as Backend>::Intermediate>) -> Result<crate::Reply<<Self as Backend>::Intermediate>>;

  /// Serializes a type `T` to the [`Intermediate`](Self::Intermediate) type.
  fn serialize<T: serde::Serialize>(from: &T) -> Result<<Self as Backend>::Intermediate>;

  /// Deserializes the [`Intermediate`](Self::Intermediate) type to a type `T`.
  fn deserialize<T>(from: &<Self as Backend>::Intermediate) -> Result<T>
  where
    T: for<'de> serde::Deserialize<'de>;
}
