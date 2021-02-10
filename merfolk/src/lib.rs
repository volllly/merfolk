#![doc(issue_tracker_base_url = "https://github.com/volllly/merfolk/issues/")]
#![doc(html_root_url = "https://github.com/volllly/merfolk")]
//#[doc(include = "../README.md")]
//#[doc = include_str!("../../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

//! [`merfolk`] is a **m**inimal **e**xtensible **r**emote procedure call **f**ramew**o**r**k**. 
//!
//! The architecture is split into three modular parts: the [`Backend`], the [`Frontend`] and optional [`Middleware`]s.
//!
//! [`merfolk`] is a collection of two things. One such thing is [`merfolk`] containing [`Mer`](crate::Mer) the orchestrator type traits and the other thing is the `Folk` a [collection](#provided-modules) of [`Backend`]s, the [`Frontend`]s and [`Middleware`]s for [`Mer`].
//!
//! [`Mer`] can act as a server or a client or both depending on the configuration.
//!
//! ## [`Backend`]
//! The Backend is responsible for sending and receiving RPCs. Depending on the [`Backend`] this can happen over different channels (e.g. http, serial port, etc.).
//! The [`Backend`] serializes and deserializes the RPCs using the [`serde`] framework.
//!
//! ## Frontend
//! The [`Frontend`] is providing an API to make RPCs and to receive them. The way RPCs are made by the client and and handled the server depend on the frontend [`Frontend`]
//!
//! ## Middleware
//! A [`Middleware`] can modify sent and received RPCs and replies. Or perform custom actions on a sent or received RPC and reply.
//!
//! # Use [`Mer`]
//! [`Mer`] needs a [`Backend`] and a [`Frontend`] to operate.
//! The following examples uses the [`Http`](/merfolk_backend_http) [`Backend`] and the [`Register`](/merfolk_frontend_register) and [`Derive`](/merfolk_frontend_derive) [`Frontend`] (see their documentation on how to use them).
//!
//! How to use [`Mer`] (how to setup the server and client) depends strongly on the used [`Frontend`].
//!
//! ## Server
//! ```
//! # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
//! # use merfolk::Mer;
//! # use merfolk_backend_http::Http;
//! # use merfolk_frontend_register::Register;
//! # fn main() {
//! // remote procedure definitions
//! fn add(a: i32, b: i32) -> i32 {
//!   a + b
//! }
//! fn subtract(a: i32, b: i32) -> i32 {
//!   a - b
//! }
//!
//! // build the backend
//! let backend = Http::builder()
//!   // configure backend as server
//!   .listen(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080))
//!   .build()
//!   .unwrap();
//!
//! // build the frontend
//! let frontend = Register::builder()
//!   .procedures(
//!     vec![
//!       ("add", Register::<Http>::make_procedure(|(a, b)| add(a, b))),
//!       ("subtract", Register::<Http>::make_procedure(|(a, b)| subtract(a, b))),
//!     ]
//!     .into_iter()
//!     .collect(),
//!   )
//!   .build()
//!   .unwrap();
//!
//! // register the procedures in the frontend
//! frontend.register("add", |(a, b)| add(a, b)).unwrap();
//! frontend.register("subtract", |(a, b)| subtract(a, b)).unwrap();
//!
//! // build merfolk instance acting as server
//! let _merfolk = Mer::builder().backend(backend).frontend(frontend).build().unwrap();
//! # }
//! ```
//!
//! ## Client
//! ```
//! # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
//! # use anyhow::Result;
//! # use merfolk::Mer;
//! # use merfolk_backend_http::Http;
//! # use merfolk_frontend_register::Register;
//! # fn main() {
//! // build the backend
//! let backend = Http::builder()
//!   // configure backend as client
//!   .speak("http://localhost:8080".parse::<hyper::Uri>().unwrap())
//!   .build()
//!   .unwrap();
//!
//! // build the frontend
//! let frontend = Register::builder().build().unwrap();
//!
//! // build merfolk instance acting as client
//! let merfolk = Mer::builder().backend(backend).frontend(frontend).build().unwrap();
//!
//! // call remote procedures via the frontend
//! let result_add: Result<i32> = merfolk.frontend(|f| f.call("add", &(1, 2))).unwrap();
//! let result_subtract: Result<i32> = merfolk.frontend(|f| f.call("subtract", &(1, 2))).unwrap();
//! # }
//! ```
//!
//! # Advanced
//! ```no_run
//! # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
//! # use merfolk::Mer;
//! # use merfolk_backend_http::Http;
//! # use merfolk_frontend_register::Register;
//! # use merfolk_frontend_derive::frontend;
//! # use merfolk_frontend_duplex::Duplex;
//! # use merfolk_middleware_router::Router;
//! # fn main() {
//! // remote procedure definitions for server
//! #[frontend()]
//! struct Receiver {}
//!
//! #[frontend(target = "Receiver")]
//! trait Definition {
//!   fn some_function(arg: String) {}
//! }
//!
//! // build the backend
//! let backend = Http::builder()
//!   // configure backend as client
//!   .speak("http://localhost:8080".parse::<hyper::Uri>().unwrap())
//!   // configure backend as server
//!   .listen(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081))
//!   .build()
//!   .unwrap();
//!
//! // build the client frontend
//! let caller_frontend = Register::builder().build().unwrap();
//!
//! // build the server frontend
//! let receiver_frontend = Receiver::builder().build().unwrap();
//!
//! // combine the frontends using the [`Duplex`](/merfolk_frontend_derive) frontend
//! let frontend = Duplex::builder().caller(caller_frontend).receiver(receiver_frontend).build().unwrap();
//!
//! // build router middleware
//! let middleware = Router::builder().routes(vec![("prefix_(.*)".to_string(), "$1".to_string())]).build_boxed().unwrap();
//!
//! // build merfolk instance acting as client and server
//! let merfolk = Mer::builder().backend(backend).frontend(frontend).middlewares(vec![middleware]).build().unwrap();
//!
//! // call remote procedures via the caller frontend
//! let result: String = merfolk.frontend(|f| f.caller.call("some_remote_function", &()).unwrap()).unwrap();
//! # }
//! ```
//!
//! # Provided Modules
//! | Type           | Name                                              | Description |
//! |----------------|---------------------------------------------------|---|
//! | [`Backend`]    | [`Http`](/merfolk_backend_http)                        | Communicates via Http and in `json` format.                                                                              |
//! | [`Backend`]    | [`InProcess`](/merfolk_backend_in_process)             | Communicates via [`tokio`](/tokio) [`channels`](https://docs.rs/tokio/1.2.0/tokio/sync/mpsc/fn.channel.html) in `json` format (mostly used for testing purposes). |
//! | [`Backend`]    | [`SerialPort`](/merfolk_backend_serialport)            | Communicates via serial port (using the [`serialport`](/serialport) library) in [`ron`](/ron) format.                                          |
//! | [`Frontend`]   | [`Derive`](/merfolk_frontend_derive)                   | Provides derive macros to derive a frontend from trait definitions.                                                      |
//! | [`Frontend`]   | [`Duplex`](/merfolk_frontend_duplex)                   | Allows for different frontends for calling and receiving RPCs.                                                            |
//! | [`Frontend`]   | [`Logger`](/merfolk_frontend_logger)                   | Provides a frontend using the [`log`](/log) facade on the client side.                                                         |
//! | [`Frontend`]   | [`Register`](/merfolk_frontend_register)                 | Allows for manually registering procedures on the server side and calling any procedure on the client side.              |
//! | [`Middleware`] | [`Authentication`](/merfolk_middleware_authentication) | Adds simple authentication and scopes.                                                                                   |
//! | [`Middleware`] | [`Router`](/merfolk_middleware_router)                 | Adds simple routing of procedures based on the procedure name.                                                           |
//!
//!
//!
//! # Develop a Module for [`Mer`] (called a `Folk`)
//! If communication over a specific channel or a different frontend etc. is needed a module can be created by implementing the [`Backend`], [`Frontend`] or [`Middleware`] trait.
//!
//! For examples please see the [provided modules](#provided-modules)
//!
//! [`Backend`]: interfaces::Backend
//! [`Frontend`]: interfaces::Frontend
//! [`Middleware`]: interfaces::Middleware
//! [`Mer`]: crate::Mer
//! [`merfolk`]: crate

extern crate alloc;

pub mod helpers;

pub mod interfaces;

#[cfg(test)]
mod test;

use anyhow::Result;

use helpers::smart_lock::SmartLock;
use log::trace;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec::Vec};

#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
/// Error type for [`Mer`] errors. Derived from `thiserror` with `std` feature.
pub enum Error {
  #[cfg_attr(feature = "std", error("mutex was poisoned"))]
  Lock,
  #[cfg_attr(feature = "std", error("could not register {source}: {source}"))]
  Register {
    #[cfg_attr(feature = "std", source)]
    source: anyhow::Error,
    end: String,
  },
  #[cfg_attr(feature = "std", error("{0} must be initialized"))]
  Init(String),
  #[cfg_attr(feature = "std", error("middleware at index {0} not found"))]
  MiddlewareIndex(usize),
  #[cfg_attr(feature = "std", error("middleware could not be downcast"))]
  DowncastError,
}

#[cfg(not(feature = "std"))]
impl From<Error> for anyhow::Error {
  fn from(e: Error) -> Self {
    anyhow::anyhow!(e)
  }
}

#[derive(Debug)]
/// Datastructure for outgoing and incoming RPC Calls.
pub struct Call<T> {
  pub procedure: String,
  pub payload: T,
}

#[derive(Debug)]
/// Data structure for outgoing and incoming RPC Replies.
pub struct Reply<T> {
  pub payload: T,
}

#[derive(derive_builder::Builder)]
#[cfg_attr(not(feature = "std"), builder(no_std))]
#[builder(pattern = "owned", build_fn(skip))]
/// RPC client and/or server type.
///
/// Container for [`Backend`](interfaces::Backend), [`Frontend`](interfaces::Frontend) and [`Middleware`](interfaces::Middleware)s.
pub struct Mer<B, F>
where
  B: interfaces::Backend,
  F: interfaces::Frontend<Backend = B>,
{
  #[builder(setter(into, name = "backend_setter"), private)]
  backend: SmartLock<B>,
  #[builder(setter(into, name = "frontend_setter"), private)]
  frontend: SmartLock<F>,

  #[allow(dead_code)]
  #[builder(setter(into, name = "middlewares_setter"), private)]
  middlewares: SmartLock<Vec<Box<dyn interfaces::Middleware<Backend = B>>>>,
}

impl<B, F> MerBuilder<B, F>
where
  B: interfaces::Backend,
  B: 'static,
  F: interfaces::Frontend<Backend = B>,
  F: 'static,
{
  pub fn backend(self, value: B) -> Self {
    self.backend_setter(smart_lock!(value))
  }

  pub fn frontend(self, value: F) -> Self {
    self.frontend_setter(smart_lock!(value))
  }

  pub fn middlewares(self, value: Vec<Box<dyn interfaces::Middleware<Backend = B>>>) -> Self {
    self.middlewares_setter(smart_lock!(value))
  }

  /// Builds a new [`Mer`].
  ///
  /// Registers the [`Backend`](interfaces::Backend), [`Frontend`](interfaces::Frontend) and [`Middleware`](interfaces::Middleware)s.
  pub fn build(self) -> Result<Mer<B, F>> {
    trace!("MerBuilder.build()");

    let backend = self.backend.ok_or_else(|| Error::Init("backend".into()))?;
    let frontend = self.frontend.ok_or_else(|| Error::Init("frontend".into()))?;
    let middlewares = self.middlewares.unwrap_or_default();

    let frontend_backend = clone_lock!(frontend);
    let middlewares_backend = clone_lock!(middlewares);

    let backend_frontend = clone_lock!(backend);
    let middlewares_frontend = clone_lock!(middlewares);

    access!(backend)
      .map_err::<anyhow::Error, _>(|_| Error::Lock.into())?
      .register(move |call: Call<B::Intermediate>| {
        trace!("Mer.backend.register()");
        let middlewares_inner = access!(middlewares_backend).map_err::<anyhow::Error, _>(|_| Error::Lock.into())?;
        let unwrapped = middlewares_inner.iter().fold(Ok(call), |acc, m| m.unwrap_call(acc));

        let reply = match unwrapped {
          Ok(unwrapped_ok) => access!(frontend_backend).map_err::<anyhow::Error, _>(|_| Error::Lock.into())?.receive(unwrapped_ok),
          Err(err) => Err(err),
        };

        middlewares_inner.iter().fold(reply, |acc, m| m.wrap_reply(acc))
      })
      .map_err::<anyhow::Error, _>(|e| Error::Register { source: e, end: "backend".into() }.into())?;

    access!(frontend)
      .map_err::<anyhow::Error, _>(|_| Error::Lock.into())?
      .register(move |call: Call<B::Intermediate>| {
        trace!("Mer.frontend.register()");
        let middlewares_inner = access!(middlewares_frontend).map_err::<anyhow::Error, _>(|_| Error::Lock.into())?;

        let wrapped = middlewares_inner.iter().rev().fold(Ok(call), |acc, m| m.wrap_call(acc));

        let reply = match wrapped {
          Ok(wrapped_ok) => access!(backend_frontend).map_err::<anyhow::Error, _>(|_| Error::Lock.into())?.call(wrapped_ok),
          Err(err) => Err(err),
        };

        middlewares_inner.iter().fold(reply, |acc, m| m.unwrap_reply(acc))
      })
      .map_err::<anyhow::Error, _>(|e| Error::Register { source: e, end: "frontend".into() }.into())?;

    Ok(Mer {
      backend: clone_lock!(backend),
      frontend: clone_lock!(frontend),
      middlewares: clone_lock!(middlewares),
    })
  }
}

/// Returns a new [`MerBuilder`]
impl<B: interfaces::Backend, F: interfaces::Frontend<Backend = B>> Mer<B, F> {
  pub fn builder() -> MerBuilder<B, F> {
    MerBuilder::default()
  }
}

impl<B: interfaces::Backend + 'static, F: interfaces::Frontend<Backend = B>> Mer<B, F> {
  /// Allows accessing the [`Frontend`](interfaces::Frontend).
  ///
  /// ```no_run
  /// # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
  /// # fn main() {
  /// #   let merfolk = merfolk::Mer::builder()
  /// #     .backend(merfolk_backend_http::Http::builder().build().unwrap())
  /// #     .frontend(merfolk_frontend_register::Register::builder().build().unwrap())
  /// #     .build()
  /// #     .unwrap();
  /// let result: i32 = merfolk.frontend(|f| f.call("add", &(1, 2)).unwrap()).unwrap();
  /// # }
  /// ```
  pub fn frontend<T, R>(&self, access: T) -> Result<R, Error>
  where
    T: Fn(&mut F) -> R,
  {
    trace!("Mer.frontend()");
    Ok(access(&mut *match access!(self.frontend) {
      Ok(frontend) => frontend,
      Err(_) => return Err(Error::Lock {}),
    }))
  }

  /// Allows accessing the [`Backend`](interfaces::Backend).
  ///
  /// ```no_run
  /// # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
  /// # fn main() {
  /// #   let merfolk = merfolk::Mer::builder()
  /// #     .backend(merfolk_backend_http::Http::builder().build().unwrap())
  /// #     .frontend(merfolk_frontend_register::Register::builder().build().unwrap())
  /// #     .build()
  /// #     .unwrap();
  /// merfolk.backend(|b| b.stop().unwrap()).unwrap();
  /// # }
  /// ```
  pub fn backend<T, R>(&self, access: T) -> Result<R, Error>
  where
    T: Fn(&mut B) -> R,
  {
    trace!("Mer.backend()");
    Ok(access(&mut *match access!(self.backend) {
      Ok(backend) => backend,
      Err(_) => return Err(Error::Lock {}),
    }))
  }

  /// Allows accessing the [`Middleware`](interfaces::Middleware)s.
  ///
  /// ```no_run
  /// # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
  /// # use merfolk_backend_http::Http;
  /// # use merfolk_middleware_authentication::Authentication;
  /// # fn main() {
  /// #   let merfolk = merfolk::Mer::builder()
  /// #     .backend(merfolk_backend_http::Http::builder().build().unwrap())
  /// #     .frontend(merfolk_frontend_register::Register::builder().build().unwrap())
  /// #     .middlewares(vec![Authentication::builder().auth(("user".into(), "pwd".into())).build_boxed().unwrap()])
  /// #     .build()
  /// #     .unwrap();
  /// merfolk.middlewares(0, |_m: &mut Authentication<Http>| {}).unwrap();
  /// # }
  /// ```
  pub fn middlewares<M, T, R>(&self, index: usize, access: T) -> Result<R, Error>
  where
    M: interfaces::Middleware<Backend = B> + 'static,
    T: Fn(&mut M) -> R,
  {
    trace!("Mer.backend()");

    let mut lock = access!(self.middlewares).map_err(|_| Error::Lock)?;

    if index >= lock.len() {
      return Err(Error::MiddlewareIndex(index));
    }

    let middleware_boxed = &mut lock[index];

    let middleware = &mut middleware_boxed.as_any().downcast_mut::<M>().ok_or(Error::DowncastError)?;

    Ok(access(middleware))
  }
}
