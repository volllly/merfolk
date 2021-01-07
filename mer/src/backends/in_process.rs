use crate::interfaces;
use snafu::{Snafu, ResultExt, OptionExt};

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

use tokio::runtime::Runtime;

pub type InProcessChannel = (crate::Call<String>, Sender<Result<crate::Reply<String>, Error>>);

#[derive(Debug, Snafu)]
pub enum Error {
  Serialize { from: serde_json::Error },
  Deserialize { from: serde_json::Error },
  GetCallLockInReceiver {},
  NoReceiver {},
  NoCallerChannel {},
  NoReceiverChannel {},
  CallerRecv { source: std::sync::mpsc::RecvError },
  CallerSend { source: std::sync::mpsc::SendError<InProcessChannel> },
  Reply { from: String },
  RuntimeCreation { source: std::io::Error }
}

pub struct InProcess {
  #[allow(clippy::type_complexity)]
  receiver: Option<Arc<dyn Fn(Arc<Mutex<crate::Call<&String>>>) -> Arc<tokio::sync::Mutex<Result<crate::Reply<String>, Error>>> + Send + Sync>>,

  runtime: Runtime,

  to: Option<Sender<InProcessChannel>>,
  from: Option<Arc<Mutex<Receiver<InProcessChannel>>>>
}

pub struct InProcessInit {
  pub to: Option<Sender<InProcessChannel>>,
  pub from: Option<Receiver<InProcessChannel>>
}

impl Default for InProcessInit {
  fn default() -> Self {
    InProcessInit {
      to: None,
      from: None
    }
  }
}

impl InProcessInit {
  pub fn init(self) -> Result<InProcess, Error> {
    Ok(InProcess {
      receiver: None,
      to: self.to,
      from: if let Some(from) = self.from { Some(Arc::new(Mutex::new(from))) } else { None },

      runtime: Runtime::new().context(RuntimeCreation)?
    })
  }
}

impl<'a> interfaces::Backend<'a> for InProcess {
  type Intermediate = String;
  type Error = Error;

  fn start(&mut self) -> Result<(), Self::Error> {
    let from = self.from.as_ref().context(NoReceiverChannel {})?.clone();
    let receiver = self.receiver.as_ref().context(NoReceiver {})?.clone();

    self.runtime.spawn(async move {
      loop {
        let (call, tx) = from.lock().unwrap().recv().unwrap();

        let reply_mutex = receiver(Arc::new(Mutex::new(crate::Call { procedure: call.procedure, payload: &call.payload })));

        let reply = &*reply_mutex.lock().await;

        tx.send(match reply {
          Ok(r) => Ok(crate::Reply { payload: r.payload.clone() }),
          Err(e) => Err(Error::Reply { from: format!("{:?}", e) })
        }).unwrap();
      }
    });

    Ok(())
  }

  fn stop(&mut self) -> Result<(), Self::Error> {
    Ok(())
  }

  fn receiver<T>(&mut self, receiver: T) -> Result<(), Self::Error>
  where
    T: Fn(&crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>, Self::Error> + Send + Sync + 'static
  {
    self.receiver = Some(Arc::new(move |call: Arc<Mutex<crate::Call<&String>>>| {
      let call = match call.as_ref().lock() {
        Ok(c) => c,
        Err(_) => return Arc::new(tokio::sync::Mutex::new(Err(Error::GetCallLockInReceiver {}))),
      };

      match receiver(&*call) {
        Ok(reply) => Arc::new(tokio::sync::Mutex::new(Ok(crate::Reply { payload: reply.payload }))),
        Err(e) => Arc::new(tokio::sync::Mutex::new(Err(e))),
      }
    }));
    Ok(())
  }

  fn call(&mut self, call: &crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>, Self::Error> {
    let (tx, rx): (Sender<Result<crate::Reply<String>, Error>>, Receiver<Result<crate::Reply<String>, Error>>) = mpsc::channel();
    self.to.as_ref().context(NoCallerChannel {})?.send((crate::Call { procedure: call.procedure.clone(), payload: call.payload.clone() }, tx)).context(CallerSend {})?;

    rx.recv().context(CallerRecv {})?
  }

  fn serialize<T: serde::Serialize>(from: &T) -> Result<String, Self::Error> {
    serde_json::to_string(from).map_err(|e| Error::Serialize { from: e })
  }

  fn deserialize<'b, T>(from: &'b Self::Intermediate) -> Result<T, Self::Error>
  where
    T: for<'de> serde::Deserialize<'de>,
  {
    serde_json::from_str(&from).map_err(|e| Error::Deserialize { from: e })
  }
}
