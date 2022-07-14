use std::{fmt::Debug, net::SocketAddr, sync::Arc};

use anyhow::Result;
use hyper::{
  client::{connect::dns::GaiResolver, HttpConnector},
  http::Uri,
  service::{make_service_fn, service_fn},
  Body, Client, Method, Request, Response, Server, StatusCode,
};
use log::{debug, info, trace};
use merfolk::{interfaces::Backend, Call, Reply};
use thiserror::Error;
use tokio::{runtime::Runtime, sync};

#[derive(Debug, Error)]
pub enum Error {
  #[error("serializing failed: {0}")]
  Serialize(#[source] serde_json::Error),
  #[error("deserializing failed {0}")]
  Deserialize(#[source] serde_json::Error),
  #[error("no speak provided in init()")]
  NoSpeak,
  #[error("error building request: {0}")]
  RequestBuilder(#[source] hyper::http::Error),
  #[error("error parsing body bytes: {0}")]
  ParseResponseBodyBytes(#[source] hyper::Error),
  #[error("error parsing body: {0}")]
  ParseResponseBody(#[from] std::string::FromUtf8Error),
  #[error("error while sending request: {0}")]
  ClientRequest(#[source] hyper::Error),
  #[error("request failed with statuscode {status}")]
  FailedRequest { status: StatusCode },
  #[error("no listen provided in init()")]
  NoListen,
  #[error("no receiver was degistered by init()")]
  NoReceiver,
  #[error("error binding server: {0}")]
  BindServer(#[from] hyper::Error),
  #[error("no procedure header in request: {0}")]
  NoProcedureHeader(#[source] hyper::http::Error),
  #[error("could not create runtime: {0}")]
  RuntimeCreation(#[from] std::io::Error),
  #[error("already started")]
  AlreadyStarted,
  #[error("not started")]
  NotStarted,
  #[error("could not send stoping message")]
  Shutdown,
}

#[derive(derive_builder::Builder)]
#[builder(pattern = "owned")]
pub struct Http {
  #[builder(setter(into, strip_option, name = "speak_setter"), private, default = "None")]
  speak: Option<(Uri, Client<HttpConnector<GaiResolver>, Body>)>,

  #[builder(setter(into, strip_option), default = "None")]
  listen: Option<SocketAddr>,

  #[allow(clippy::type_complexity)]
  #[builder(private, default = "None")]
  receiver: Option<Arc<dyn Fn(Call<String>) -> Result<Reply<String>> + Send + Sync>>,

  #[builder(private, default = "Runtime::new().map_err(Error::RuntimeCreation).map_err(|e| e.to_string())?")]
  runtime: Runtime,

  #[builder(private, default = "None")]
  shutdown: Option<sync::oneshot::Sender<()>>,
}

impl HttpBuilder {
  pub fn speak(self, value: Uri) -> Self {
    self.speak_setter((value, Client::new()))
  }
}

impl Http {
  pub fn builder() -> HttpBuilder {
    HttpBuilder::default()
  }
}

impl Debug for Http {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    f.debug_struct("Http")
      .field("speak", &self.speak)
      .field("listen", &self.listen)
      .field("runtime", &self.runtime)
      .finish()
  }
}

impl Http {
  pub fn start(&mut self) -> Result<()> {
    trace!("start Http Backend");

    if self.shutdown.is_some() {
      return Err(Error::AlreadyStarted.into());
    }

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    self.shutdown = Some(tx);

    let listen = self.listen.ok_or(Error::NoListen)?;

    let receiver = Arc::clone(self.receiver.as_ref().ok_or(Error::NoReceiver)?);

    self.runtime.spawn(async move {
      trace!("spawn listener");

      let receiver = receiver.clone();

      Server::bind(&listen)
        .serve(make_service_fn(move |_| {
          trace!("serve connection");

          let receiver = receiver.clone();
          async move {
            Ok::<_, hyper::Error>(service_fn(move |request: Request<Body>| {
              trace!("run service_fn");

              debug!("{:?}", &listen);

              info!("received incomming call");

              let receiver = receiver.clone();
              async move {
                let procedure = if let Some(procedure) = request.headers().get("Procedure") {
                  match procedure.to_str() {
                    Ok(procedure) => procedure.to_owned(),
                    Err(e) => return Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())),
                  }
                } else {
                  return Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from("No Procedure provided"));
                };

                let body_bytes = match hyper::body::to_bytes(request.into_body()).await {
                  Ok(body_bytes) => body_bytes,
                  Err(e) => return Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())),
                };

                let body = match String::from_utf8(body_bytes.to_vec()) {
                  Ok(body) => body,
                  Err(e) => return Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())),
                };

                debug!("call Call {{ procedure: {:?}, payload: {:?} }}", &procedure, &body);
                let reply = receiver(crate::Call { procedure, payload: body });

                match reply {
                  Err(e) => Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(format!("{:?}", e))),

                  Ok(reply) => {
                    debug!("reply Reply {{ payload: {:?} }}", &reply.payload);
                    Response::builder().status(StatusCode::OK).body(Body::from(reply.payload))
                  }
                }
              }
            }))
          }
        }))
        .with_graceful_shutdown(async {
          rx.await.ok();
        })
        .await
        .unwrap();
    });
    Ok(())
  }

  pub fn stop(&mut self) -> Result<()> {
    trace!("stop http backend");
    self.shutdown.take().ok_or(Error::NotStarted)?.send(()).map_err(|_| Error::Shutdown.into())
  }
}

impl Backend for Http {
  type Intermediate = String;

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

    if let Some(err) = self.start().err().map(|e| e.downcast::<Error>()) {
      match err {
        Ok(err) => match err {
          Error::AlreadyStarted | Error::NoListen | Error::NoReceiver => {}
          err => return Err(err.into()),
        },
        Err(err) => return Err(err),
      }
    };

    Ok(())
  }

  fn call(&mut self, call: Call<Self::Intermediate>) -> Result<Reply<Self::Intermediate>> {
    trace!("call backend");

    debug!("{:?}", &self.speak);

    info!("received outgoing call");

    match &self.speak {
      None => Err(Error::NoSpeak.into()),

      Some(speak) => self.runtime.block_on(async {
        let request = Request::builder()
          .method(Method::POST)
          .uri(&speak.0)
          .header("Procedure", &call.procedure)
          .body(Body::from(call.payload.clone()))
          .map_err(Error::RequestBuilder)?;

        debug!("request {:?}", &request);
        let response = speak.1.request(request).await.map_err(Error::ClientRequest)?;
        debug!("response {:?}", &response);

        let status = response.status();
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.map_err(Error::ParseResponseBodyBytes)?;
        let body = String::from_utf8(body_bytes.to_vec()).map_err(Error::ParseResponseBody)?;

        match status {
          StatusCode::OK => Ok(Reply { payload: body }),
          _ => Err(Error::FailedRequest { status }.into()),
        }
      }),
    }
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

    serde_json::from_str(from).map_err(|e| Error::Deserialize(e).into())
  }
}

impl Drop for Http {
  fn drop(&mut self) {
    if self.shutdown.is_some() {
      self.stop().unwrap()
    }
  }
}
