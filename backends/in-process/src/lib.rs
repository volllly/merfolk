use mer::{interfaces::Backend, Call, Reply};

use snafu::{OptionExt, ResultExt, Snafu};

use std::sync::{Arc, Mutex};

use tokio::runtime::Runtime;

use tokio::sync::{
  mpsc,
  mpsc::{Receiver, Sender},
};

use log::trace;

pub type InProcessChannel = (Call<String>, Sender<Result<Reply<String>, Error>>);

#[derive(Debug, Snafu)]
pub enum Error {
  Serialize { from: serde_json::Error },
  Deserialize { from: serde_json::Error },
  GetCallLockInReceiver {},
  NoReceiver {},
  NoCallerChannel {},
  NoReceiverChannel {},
  CallerRecv {},
  CallerSend { source: tokio::sync::mpsc::error::SendError<InProcessChannel> },
  ReplyError { from: String },
  RuntimeCreation { source: std::io::Error },
  AlreadyStarted,
  NotStarted,
}

pub struct InProcess {
  #[allow(clippy::type_complexity)]
  receiver: Option<Arc<dyn Fn(Arc<Mutex<Call<&String>>>) -> Arc<tokio::sync::Mutex<Result<Reply<String>, Error>>> + Send + Sync>>,

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
  pub fn init(self) -> Result<InProcess, Error> {
    trace!("initialze InProcessInit");

    Ok(InProcess {
      receiver: None,
      to: self.to,
      from: if let Some(from) = self.from { Some(Arc::new(tokio::sync::Mutex::new(from))) } else { None },

      runtime: Runtime::new().context(RuntimeCreation)?,
      handle: None,
    })
  }
}

impl<'a> Backend<'a> for InProcess {
  type Intermediate = String;
  type Error = Error;

  fn start(&mut self) -> Result<(), Self::Error> {
    trace!("start InProcessInit");

    if self.handle.is_some() {
      return Err(Error::AlreadyStarted);
    }

    let from = self.from.as_ref().context(NoReceiverChannel {})?.clone();
    let receiver = self.receiver.as_ref().context(NoReceiver {})?.clone();

    self.handle = Some(self.runtime.spawn(async move {
      loop {
        let (call, tx) = from.lock().await.recv().await.unwrap();

        let reply_mutex = receiver(Arc::new(Mutex::new(Call {
          procedure: call.procedure,
          payload: &call.payload,
        })));

        let reply = &*reply_mutex.lock().await;

        tx.send(match reply {
          Ok(r) => Ok(Reply { payload: r.payload.clone() }),
          Err(e) => Err(Error::ReplyError { from: format!("{:?}", e) }),
        })
        .await
        .unwrap();
      }
    }));

    Ok(())
  }

  fn stop(&mut self) -> Result<(), Self::Error> {
    trace!("stop InProcessInit");

    match &self.handle {
      None => Err(Error::NotStarted),
      Some(handle) => {
        handle.abort();
        Ok(())
      }
    }
  }

  fn receiver<T>(&mut self, receiver: T) -> Result<(), Self::Error>
  where
    T: Fn(&Call<&Self::Intermediate>) -> Result<Reply<Self::Intermediate>, Self::Error> + Send + Sync + 'static,
  {
    trace!("register receiver");

    self.receiver = Some(Arc::new(move |call: Arc<Mutex<Call<&String>>>| {
      let call = match call.as_ref().lock() {
        Ok(c) => c,
        Err(_) => return Arc::new(tokio::sync::Mutex::new(Err(Error::GetCallLockInReceiver {}))),
      };

      match receiver(&*call) {
        Ok(reply) => Arc::new(tokio::sync::Mutex::new(Ok(Reply { payload: reply.payload }))),
        Err(e) => Arc::new(tokio::sync::Mutex::new(Err(e))),
      }
    }));
    Ok(())
  }

  fn call(&mut self, call: &Call<&Self::Intermediate>) -> Result<Reply<Self::Intermediate>, Self::Error> {
    trace!("receive call");

    #[allow(clippy::type_complexity)]
    self.runtime.block_on(async {
      let (tx, mut rx): (Sender<Result<Reply<String>, Error>>, Receiver<Result<Reply<String>, Error>>) = mpsc::channel(1);
      self
        .to
        .as_ref()
        .context(NoCallerChannel {})?
        .send((
          Call {
            procedure: call.procedure.clone(),
            payload: call.payload.clone(),
          },
          tx,
        ))
        .await
        .context(CallerSend {})?;

      rx.recv().await.context(CallerRecv {})?
    })
  }

  fn serialize<T: serde::Serialize>(from: &T) -> Result<String, Self::Error> {
    trace!("serialize from");

    serde_json::to_string(from).map_err(|e| Error::Serialize { from: e })
  }

  fn deserialize<'b, T>(from: &'b Self::Intermediate) -> Result<T, Self::Error>
  where
    T: for<'de> serde::Deserialize<'de>,
  {
    trace!("deserialize from");

    serde_json::from_str(&from).map_err(|e| Error::Deserialize { from: e })
  }
}

impl Drop for InProcess {
  fn drop(&mut self) {
    self.stop().unwrap()
  }
}
