use mer::{interfaces::Backend, Call, Reply};

use snafu::{OptionExt, ResultExt, Snafu};

use hyper::{
  client::{connect::dns::GaiResolver, HttpConnector},
  Body, Client, Method, Request, StatusCode,
};

use hyper::http::Uri;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Response, Server};
use std::sync::{Arc, Mutex};
use std::{fmt::Debug, net::SocketAddr};
use tokio::runtime::Runtime;
use tokio::sync;

use log::{info, debug, trace};

#[derive(Debug, Snafu)]
pub enum Error {
  Serialize { from: serde_json::Error },
  Deserialize { from: serde_json::Error },
  SpeakingDisabled,
  RequestBuilder { source: hyper::http::Error },
  ParseResponseBodyBytes { source: hyper::Error },
  ParseResponseBody { source: std::string::FromUtf8Error },
  ClientRequest { source: hyper::Error },
  FailedRequest { status: StatusCode },
  NoListen,
  NoReceiver,
  BindServer { source: hyper::Error },
  NoProcedureHeader { source: hyper::http::Error },
  GetCallLockInReceiver,
  RuntimeCreation { source: std::io::Error },
  AlreadyStarted,
  NotStarted,
  Shutdown { }
}
pub struct Http {
  client: Client<HttpConnector<GaiResolver>, Body>,
  speak: Option<Uri>,
  listen: Option<SocketAddr>,
  #[allow(clippy::type_complexity)]
  receiver: Option<Arc<dyn Fn(Arc<Mutex<Call<&String>>>) -> Arc<sync::Mutex<Result<Reply<String>, Error>>> + Send + Sync>>,
  runtime: Runtime,

  shutdown: Option<sync::oneshot::Sender<()>>
}

impl Debug for Http {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    f.debug_struct("Http")
      .field("client", &self.client)
      .field("speak", &self.speak)
      .field("listen", &self.listen)
      .field("runtime", &self.runtime)
      .finish()
  }
}

pub struct HttpInit {
  pub client: Client<HttpConnector<GaiResolver>, Body>,
  pub speak: Option<Uri>,
  pub listen: Option<SocketAddr>,
}

impl Default for HttpInit {
  fn default() -> Self {
    HttpInit {
      client: Client::new(),
      speak: None,
      listen: None,
    }
  }
}

impl From<HttpInit> for Result<Http, Error> {
  fn from(from: HttpInit) -> Self {
    from.init()
  }
}

impl HttpInit {
  pub fn init(self) -> Result<Http, Error> {
    trace!("initialize HttpInit");

    let http = Http {
      client: self.client,
      speak: self.speak,
      listen: self.listen,
      receiver: None,
      shutdown: None,
      runtime: Runtime::new().context(RuntimeCreation {})?,
    };

    debug!("{:?}", &http);

    Ok(http)
  }
}

impl Backend<'_> for Http {
  type Intermediate = String;
  type Error = Error;

  fn start(&mut self) -> Result<(), Self::Error> {
    trace!("start Http Backend");

    if self.shutdown.is_some() {
      return Err(Error::AlreadyStarted);
    }

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    self.shutdown = Some(tx);

    let listen = self.listen.context(NoListen)?;

    let receiver = Arc::clone(self.receiver.as_ref().context(NoReceiver)?);

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
                let reply_mutex = receiver(Arc::new(Mutex::new(crate::Call { procedure, payload: &body })));

                let reply = &*reply_mutex.lock().await;

                match reply {
                  Err(e) => Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(format!("{:?}", e))),

                  Ok(reply) => {
                    debug!("reply Reply {{ payload: {:?} }}", &reply.payload);
                    Response::builder().status(StatusCode::OK).body(Body::from(reply.payload.to_owned()))
                  }
                }
              }
            }))
          }
        }))
        .with_graceful_shutdown(async { rx.await.ok(); })
        .await
        .unwrap();
    });
    Ok(())
  }

  fn stop(&mut self) -> Result<(), Self::Error> {
    trace!("stop http backend");
    match self.shutdown.take() {
      None => Err(Error::NotStarted),
      Some(tx) => tx.send(()).map_err(|_| Error::Shutdown {})
    }
  }

  fn receiver<T>(&mut self, receiver: T) -> Result<(), Self::Error>
  where
    T: Fn(&Call<&Self::Intermediate>) -> Result<Reply<Self::Intermediate>, Self::Error> + Send + Sync + 'static,
  {
    trace!("register receiver");

    self.receiver = Some(Arc::new(move |call: Arc<Mutex<Call<&String>>>| {
      trace!("run receiver");

      let call = match call.as_ref().lock() {
        Ok(c) => c,
        Err(_) => return Arc::new(sync::Mutex::new(Err(Error::GetCallLockInReceiver))),
      };

      debug!("calling receiver");
      match receiver(&*call) {
        Ok(reply) => Arc::new(sync::Mutex::new(Ok(Reply { payload: reply.payload }))),
        Err(e) => Arc::new(sync::Mutex::new(Err(e))),
      }
    }));

    Ok(())
  }

  fn call(&mut self, call: &Call<&Self::Intermediate>) -> Result<Reply<Self::Intermediate>, Self::Error> {
    trace!("call backend");

    debug!("{:?}", &self.speak);

    info!("received outgoing call");

    match &self.speak {
      None => Err(Error::SpeakingDisabled),

      Some(uri) => self.runtime.block_on(async {
        let request = Request::builder()
          .method(Method::POST)
          .uri(uri)
          .header("Procedure", &call.procedure)
          .body(Body::from(call.payload.clone()))
          .context(RequestBuilder)?;

        debug!("request {:?}", &request);
        let response = self.client.request(request).await.context(ClientRequest)?;
        debug!("response {:?}", &response);

        let status = response.status();
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.context(ParseResponseBodyBytes)?;
        let body = String::from_utf8(body_bytes.to_vec()).context(ParseResponseBody)?;

        match status {
          StatusCode::OK => Ok(Reply { payload: body }),
          _ => Err(Error::FailedRequest { status }),
        }
      }),
    }
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


impl Drop for Http {
  fn drop(&mut self) {
      self.stop().unwrap()
  }
}