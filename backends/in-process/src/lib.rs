use mer::{interfaces::Backend, Call, Reply};

use snafu::{OptionExt, ResultExt, Snafu};

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use tokio::runtime::Runtime;

pub type InProcessChannel = (Call<String>, Sender<Result<Reply<String>, Error>>);

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
  ReplyError { from: String },
  RuntimeCreation { source: std::io::Error },
}

pub struct InProcess {
  #[allow(clippy::type_complexity)]
  receiver: Option<Arc<dyn Fn(Arc<Mutex<Call<&String>>>) -> Arc<tokio::sync::Mutex<Result<Reply<String>, Error>>> + Send + Sync>>,

  runtime: Runtime,

  to: Option<Sender<InProcessChannel>>,
  from: Option<Arc<Mutex<Receiver<InProcessChannel>>>>,
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
    Ok(InProcess {
      receiver: None,
      to: self.to,
      from: if let Some(from) = self.from { Some(Arc::new(Mutex::new(from))) } else { None },

      runtime: Runtime::new().context(RuntimeCreation)?,
    })
  }
}

impl<'a> Backend<'a> for InProcess {
  type Intermediate = String;
  type Error = Error;

  fn start(&mut self) -> Result<(), Self::Error> {
    let from = self.from.as_ref().context(NoReceiverChannel {})?.clone();
    let receiver = self.receiver.as_ref().context(NoReceiver {})?.clone();

    self.runtime.spawn(async move {
      loop {
        let (call, tx) = from.lock().unwrap().recv().unwrap();

        let reply_mutex = receiver(Arc::new(Mutex::new(Call {
          procedure: call.procedure,
          payload: &call.payload,
        })));

        let reply = &*reply_mutex.lock().await;

        tx.send(match reply {
          Ok(r) => Ok(Reply { payload: r.payload.clone() }),
          Err(e) => Err(Error::ReplyError { from: format!("{:?}", e) }),
        })
        .unwrap();
      }
    });

    Ok(())
  }

  fn stop(&mut self) -> Result<(), Self::Error> {
    Ok(())
  }

  fn receiver<T>(&mut self, receiver: T) -> Result<(), Self::Error>
  where
    T: Fn(&Call<&Self::Intermediate>) -> Result<Reply<Self::Intermediate>, Self::Error> + Send + Sync + 'static,
  {
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
    #[allow(clippy::type_complexity)]
    let (tx, rx): (Sender<Result<Reply<String>, Error>>, Receiver<Result<Reply<String>, Error>>) = mpsc::channel();
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
      .context(CallerSend {})?;

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
