use mer::{interfaces::Backend, Call, Reply};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::{fmt::Debug, sync::Arc};
use tokio::{runtime::Runtime, sync::Mutex};

use log::{debug, error, info, trace};

#[derive(Debug, Error)]
pub enum Error {
  #[error("serializing failed: {0}")]
  Serialize(#[source] serde_json::Error),
  #[error("deserializing failed: {0}")]
  Deserialize(#[source] serde_json::Error),
  #[error("no receiver was degistered by init()")]
  NoReceiver,
  #[error("could not create runtime: {0}")]
  RuntimeCreation(#[from] std::io::Error),
  #[error("already started")]
  AlreadyStarted,
  #[error("not started")]
  NotStarted,
  #[error("error while sending: {0}")]
  SendError(#[source] std::io::Error),
  #[error("no sender channel still alive")]
  NoSenderChannel,
  #[error("from frontend: {0}")]
  FromFrontend(#[source] anyhow::Error),
}

pub struct SerialPort {
  port: Arc<Mutex<Box<dyn serialport::SerialPort>>>,

  #[allow(clippy::type_complexity)]
  receiver: Option<Arc<dyn Fn(Call<String>) -> Result<Reply<String>> + Send + Sync>>,

  reply_queue: Option<Arc<Mutex<tokio::sync::mpsc::Receiver<String>>>>,

  runtime: Runtime,

  handle: Option<tokio::task::JoinHandle<std::convert::Infallible>>,
}

pub struct SerialPortInit {
  pub port: Box<dyn serialport::SerialPort>,
}

impl From<SerialPortInit> for Result<SerialPort> {
  fn from(from: SerialPortInit) -> Self {
    from.init()
  }
}

impl SerialPortInit {
  pub fn init(self) -> Result<SerialPort> {
    trace!("initialize SerialPortInit");

    debug!("port: {:?}", self.port.name());

    Ok(SerialPort {
      port: Arc::new(Mutex::new(self.port)),
      receiver: None,
      reply_queue: None,
      handle: None,
      runtime: Runtime::new().map_err(Error::RuntimeCreation)?,
    })
  }
}

#[derive(Serialize, Deserialize)]
struct SelfCall {
  procedure: String,
  payload: String,
}

#[derive(Serialize, Deserialize)]
struct SelfReply {
  payload: String,
}

impl Backend for SerialPort {
  type Intermediate = String;

  fn start(&mut self) -> Result<()> {
    trace!("start SerialPort Backend");

    if self.handle.is_some() {
      return Err(Error::AlreadyStarted.into());
    }

    let receiver = Arc::clone(self.receiver.as_ref().ok_or(Error::NoReceiver)?);

    let (tx, rx) = tokio::sync::mpsc::channel::<String>(2);

    self.reply_queue = Some(Arc::new(Mutex::new(rx)));

    let port = Arc::clone(&self.port);

    self.handle = Some(self.runtime.spawn(async move {
      trace!("spawn listener");

      loop {
        trace!("reading serialport");

        let mut read: Vec<u8> = vec![];

        let mut port_gate = port.lock().await;

        loop {
          let mut buf: Vec<u8> = vec![0; 1024];

          match port_gate.read(buf.as_mut_slice()) {
            Ok(n) => {
              debug!("{} read {} bytes", port_gate.name().unwrap_or_else(|| "".to_string()), n);
              read.append(&mut buf[0..n].to_vec());

              if n != buf.len() {
                break;
              }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
              debug!("{} read timeout", port_gate.name().unwrap_or_else(|| "".to_string()));
              break;
            }
            Err(e) => {
              error!("{:?}", e);
              break;
            }
          }
        }

        if !read.is_empty() {
          if let Ok(read_string) = String::from_utf8(read) {
            let read_parts = read_string.split("\r\n");

            for part in read_parts {
              if part.is_empty() {
                break;
              }

              match &part[0..2] {
                "r:" => {
                  debug!("{} read reply", port_gate.name().unwrap_or_else(|| "".to_string()));

                  tx.send(part[2..].to_string()).await.unwrap()
                }
                "c:" => {
                  debug!("{} read call", port_gate.name().unwrap_or_else(|| "".to_string()));

                  let read_unpacked = part[2..].to_string();
                  let self_call: SelfCall = Self::deserialize(&read_unpacked).unwrap();
                  let reply = receiver(Call {
                    procedure: self_call.procedure,
                    payload: self_call.payload,
                  });

                  let self_reply = match reply.map(|r| SelfReply { payload: r.payload }) {
                    Ok(ok) => std::result::Result::Ok(ok),
                    Err(err) => std::result::Result::Err(err.to_string()),
                  };
                  let self_reply_string = "r:".to_string() + &Self::serialize(&self_reply).unwrap() + "\r\n";

                  for _ in 0..2 {
                    match port_gate.write(&self_reply_string.as_bytes()) {
                      Ok(n) => {
                        debug!("{} sent r: {} bytes", port_gate.name().unwrap_or_else(|| "".to_string()), n);
                        break;
                      }
                      Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                      Err(e) => log::error!("{:?}", e),
                    }
                  }
                }
                _ => {}
              }
            }
          };
        }
      }
    }));
    Ok(())
  }

  fn stop(&mut self) -> Result<()> {
    trace!("stop serialport backend");
    match &self.handle {
      None => Err(Error::NotStarted.into()),
      Some(handle) => {
        handle.abort();
        Ok(())
      }
    }
  }

  fn register<T>(&mut self, receiver: T) -> Result<()>
  where
    T: Fn(Call<Self::Intermediate>) -> Result<Reply<Self::Intermediate>> + Send + Sync + 'static,
  {
    trace!("register receiver");

    self.receiver = Some(Arc::new(move |call: Call<String>| {
      trace!("run receiver");

      debug!("calling receiver");
      receiver(call)
    }));

    Ok(())
  }

  fn call(&mut self, call: Call<Self::Intermediate>) -> Result<Reply<Self::Intermediate>> {
    trace!("call backend");

    info!("received outgoing call");

    if self.reply_queue.is_none() {
      return Err(Error::NotStarted.into());
    }

    let port = Arc::clone(&self.port);
    let reply_queue = Arc::clone(&self.reply_queue.as_ref().unwrap());

    self.runtime.block_on(async move {
      let self_call = SelfCall {
        procedure: call.procedure,
        payload: call.payload,
      };
      let self_call_string = "c:".to_string() + &Self::serialize(&self_call).unwrap() + "\r\n";

      let port_name;
      let written;
      {
        let mut port_gate = port.lock().await;

        port_name = port_gate.name().unwrap_or_else(|| "".to_string());

        written = port_gate.write(&self_call_string.as_bytes());
      }

      match written {
        Ok(n) => {
          debug!("{} sent c: {} bytes", port_name, n);
          let mut queue_lock = reply_queue.lock().await;

          match queue_lock.recv().await {
            Some(self_reply_string) => Ok(Reply {
              payload: Self::deserialize::<Result<SelfReply, String>>(&self_reply_string)?
                .map_err(|e| Error::FromFrontend(anyhow::anyhow!(e)))?
                .payload,
            }),
            None => Err(Error::NoSenderChannel.into()),
          }
        }
        Err(e) => Err(Error::SendError(e).into()),
      }
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

impl Drop for SerialPort {
  fn drop(&mut self) {
    if self.handle.is_some() {
      self.stop().unwrap()
    }
  }
}
