use mer::{interfaces::Backend, Call, Reply};

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

pub struct InProcess {
  #[allow(clippy::type_complexity)]
  receiver: Option<Arc<dyn Fn(Call<String>) -> Result<Reply<String>> + Send + Sync>>,

  runtime: Runtime,

  handle: Option<tokio::task::JoinHandle<std::convert::Infallible>>,

  to: Option<Sender<InProcessChannel>>,
  from: Option<Arc<tokio::sync::Mutex<Receiver<InProcessChannel>>>>,
}

pub struct InProcessInit {
  pub to: Option<Sender<InProcessChannel>>,
  pub from: Option<Receiver<InProcessChannel>>,
}

impl Default for InProcessInit {
  fn default() -> Self {
    InProcessInit { to: None, from: None }
  }
}

impl InProcessInit {
  pub fn init(self) -> Result<InProcess> {
    trace!("initialze InProcessInit");

    Ok(InProcess {
      receiver: None,
      to: self.to,
      from: if let Some(from) = self.from { Some(Arc::new(tokio::sync::Mutex::new(from))) } else { None },

      runtime: Runtime::new().map_err(Error::RuntimeCreation)?,
      handle: None,
    })
  }
}

impl Backend for InProcess {
  type Intermediate = String;

  fn start(&mut self) -> Result<()> {
    trace!("start InProcessInit");

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

  fn stop(&mut self) -> Result<()> {
    trace!("stop InProcessInit");

    match &self.handle {
      None => Err(Error::NotStarted.into()),
      Some(handle) => {
        handle.abort();
        Ok(())
      }
    }
  }

  fn receiver<T>(&mut self, receiver: T) -> Result<()>
  where
    T: Fn(Call<Self::Intermediate>) -> Result<Reply<Self::Intermediate>> + Send + Sync + 'static,
  {
    trace!("register receiver");

    self.receiver = Some(Arc::new(move |call: Call<String>| receiver(call)));
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
