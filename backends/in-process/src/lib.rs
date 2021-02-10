use merfolk::{interfaces::Backend, Call, Reply};

use anyhow::Result;
use thiserror::Error;

use std::sync::Arc;

use tokio::runtime::Runtime;

use tokio::sync::{
  mpsc,
  mpsc::{Receiver, Sender},
};

use log::trace;

pub type InProcessChannel = (Call<String>, Sender<Result<Reply<String>>>);

#[derive(Debug, Error)]
pub enum Error {
  #[error("derializing failed: {0}")]
  Serialize(#[source] serde_json::Error),
  #[error("deserializing failed: {0}")]
  Deserialize(#[source] serde_json::Error),
  #[error("no receiver was registered by the init() function")]
  NoReceiver,
  #[error("no `to` channel was provided in init()")]
  NoCallerChannel,
  #[error("no `from` channel was provided in init()")]
  NoReceiverChannel,
  #[error("recv() from `rx` channel failed")]
  CallerRecv,
  #[error("send() to `to` channel failed: {0}")]
  CallerSend(#[from] tokio::sync::mpsc::error::SendError<InProcessChannel>),
  #[error("could not create runtime: {0}")]
  RuntimeCreation(#[from] std::io::Error),
  #[error("already started")]
  AlreadyStarted,
  #[error("not yet started")]
  NotStarted,
}

#[derive(derive_builder::Builder)]
#[builder(pattern = "owned")]
pub struct InProcess {
  #[allow(clippy::type_complexity)]
  #[builder(private, default = "None")]
  receiver: Option<Arc<dyn Fn(Call<String>) -> Result<Reply<String>> + Send + Sync>>,

  #[builder(private, default = "Runtime::new().map_err(Error::RuntimeCreation).map_err(|e| e.to_string())?")]
  runtime: Runtime,

  #[builder(private, default = "None")]
  handle: Option<tokio::task::JoinHandle<std::convert::Infallible>>,

  #[builder(setter(into, strip_option), default = "None")]
  to: Option<Sender<InProcessChannel>>,

  #[builder(setter(into, strip_option, name = "from_setter"), private, default = "None")]
  from: Option<Arc<tokio::sync::Mutex<Receiver<InProcessChannel>>>>,
}

impl InProcessBuilder {
  pub fn from(self, value: Receiver<InProcessChannel>) -> Self {
    self.from_setter(Arc::new(tokio::sync::Mutex::new(value)))
  }
}

impl InProcess {
  pub fn builder() -> InProcessBuilder {
    InProcessBuilder::default()
  }
}

impl InProcess {
  pub fn start(&mut self) -> Result<()> {
    trace!("start InProcess");

    if self.handle.is_some() {
      return Err(Error::AlreadyStarted.into());
    }

    let from = self.from.as_ref().ok_or(Error::NoReceiverChannel)?.clone();
    let receiver = self.receiver.as_ref().ok_or(Error::NoReceiver)?.clone();

    self.handle = Some(self.runtime.spawn(async move {
      loop {
        let (call, tx) = from.lock().await.recv().await.unwrap();

        let reply = receiver(Call {
          procedure: call.procedure,
          payload: call.payload,
        });

        tx.send(reply).await.unwrap();
      }
    }));

    Ok(())
  }

  pub fn stop(&mut self) -> Result<()> {
    trace!("stop InProcess");

    match &self.handle {
      None => Err(Error::NotStarted.into()),
      Some(handle) => {
        handle.abort();
        Ok(())
      }
    }
  }
}

impl Backend for InProcess {
  type Intermediate = String;

  fn register<T>(&mut self, receiver: T) -> Result<()>
  where
    T: Fn(Call<Self::Intermediate>) -> Result<Reply<Self::Intermediate>> + Send + Sync + 'static,
  {
    trace!("register receiver");

    self.receiver = Some(Arc::new(move |call: Call<String>| receiver(call)));

    self.start().ok();

    Ok(())
  }

  fn call(&mut self, call: Call<Self::Intermediate>) -> Result<Reply<Self::Intermediate>> {
    trace!("receive call");

    self.runtime.block_on(async {
      #[allow(clippy::type_complexity)]
      let (tx, mut rx): (Sender<Result<Reply<String>>>, Receiver<Result<Reply<String>>>) = mpsc::channel(1);
      self
        .to
        .as_ref()
        .ok_or(Error::NoCallerChannel)?
        .send((
          Call {
            procedure: call.procedure.clone(),
            payload: call.payload.clone(),
          },
          tx,
        ))
        .await
        .map_err(Error::CallerSend)?;

      rx.recv().await.ok_or(Error::CallerRecv)?
    })
  }

  fn serialize<T: serde::Serialize>(from: &T) -> Result<String> {
    trace!("serialize from");

    serde_json::to_string(from).map_err(|e| Error::Serialize(e).into())
  }

  fn deserialize<'b, T>(from: &'b Self::Intermediate) -> Result<T>
  where
    T: for<'de> serde::Deserialize<'de>,
  {
    trace!("deserialize from");

    serde_json::from_str(&from).map_err(|e| Error::Deserialize(e).into())
  }
}

impl Drop for InProcess {
  fn drop(&mut self) {
    self.stop().unwrap()
  }
}
