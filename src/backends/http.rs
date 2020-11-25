use crate::interfaces;
use crate::interfaces::backend::{Error, Result};

use snafu::{ResultExt, Snafu};

use hyper::{
  client::{connect::dns::GaiResolver, HttpConnector},
  Body, Client, Method, Request, StatusCode,
};

use hyper::http::Uri;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Response, Server};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::sync;

#[derive(Debug, Snafu)]
pub enum LocalError {
  RequestBuilder { source: hyper::Error },
}

pub struct Http {
  client: Client<HttpConnector<GaiResolver>, Body>,
  speak: Option<Uri>,
  listen: Option<SocketAddr>,
  #[allow(clippy::type_complexity)]
  receiver: Option<Arc<sync::Mutex<dyn Fn(Arc<Mutex<crate::Call<&String>>>) -> Arc<sync::Mutex<Result<crate::Reply<String>>>> + Send>>>,
  runtime: Runtime,
}

impl Default for Http {
  fn default() -> Self {
    Http {
      client: Client::new(),
      speak: None,
      listen: None,
      receiver: None,
      runtime: Runtime::new().unwrap(),
    }
  }
}

impl Http {
  pub fn new(speak: Option<Uri>, listen: Option<SocketAddr>) -> Http {
    Http { speak, listen, ..Http::default() }
  }
}

impl<'a> interfaces::Backend<'a> for Http {
  type Intermediate = String;

  fn start(&mut self) -> Result<()> {
    let listen = self.listen.unwrap();
    let receiver = Arc::clone(self.receiver.as_ref().unwrap());

    self.runtime.spawn(async move {
      let receiver = receiver.clone();

      Server::bind(&listen)
        .serve(make_service_fn(move |_| {
          let receiver = receiver.clone();
          async move {
            Ok::<_, hyper::Error>(service_fn(move |request: Request<Body>| {
              let receiver = receiver.clone();
              async move {
                let procedure = if let Some(procedure) = request.headers().get("Procedure") {
                  match procedure.to_str() {
                    Ok(procedure) => procedure.to_owned(),
                    Err(e) => return Ok::<_, hyper::Error>(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap()),
                  }
                } else {
                  return Ok::<_, hyper::Error>(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from("No Procedure provided")).unwrap());
                };

                let body_bytes = hyper::body::to_bytes(request.into_body()).await.unwrap();
                let body = match String::from_utf8(body_bytes.to_vec()) {
                  Ok(body) => body,
                  Err(e) => return Ok::<_, hyper::Error>(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap()),
                };

                let reply_mutex = receiver.lock().await(Arc::new(Mutex::new(crate::Call { procedure, payload: &body })));

                let reply = &*reply_mutex.lock().await;

                match reply {
                  Err(e) => Ok::<_, hyper::Error>(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(format!("{:?}", e))).unwrap()),

                  Ok(reply) => Ok::<_, hyper::Error>(Response::builder().status(StatusCode::OK).body(Body::from(reply.payload.to_owned())).unwrap()),
                }
              }
            }))
          }
        }))
        .await
        .unwrap();
    });
    Ok(())
  }

  fn stop(&mut self) -> Result<()> {
    Ok(())
  }

  fn receiver<T>(&mut self, receiver: T) -> Result<()>
  where
    T: Fn(&crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>> + Send,
    T: 'static,
  {
    self.receiver = Some(Arc::new(sync::Mutex::new(move |call: Arc<Mutex<crate::Call<&String>>>| match receiver(&*call.lock().unwrap()) {
      Ok(reply) => Arc::new(sync::Mutex::new(Ok(crate::Reply { payload: reply.payload }))),
      Err(e) => Arc::new(sync::Mutex::new(Err(e))),
    })));

    Ok(())
  }

  fn call(&mut self, call: &crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>> {
    match &self.speak {
      None => Err(Error::SpeakingDisabled),

      Some(uri) => self.runtime.block_on(async {
        let request = Request::builder()
          .method(Method::POST)
          .uri(uri)
          .header("Procedure", &call.procedure)
          .body(Body::from(call.payload.clone()))
          .map_err(|e| Error::Call { inner: e.to_string() })?;

        let result = self.client.request(request).await;

        let response = result.map_err(|e| Error::Call { inner: e.to_string() })?;
        let status = response.status();
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body = String::from_utf8(body_bytes.to_vec());

        match status {
          StatusCode::OK => Ok(crate::Reply { payload: body.unwrap() }),
          _ => Err(Error::Call {
            inner: format!("invalid statuscode: {} returned", status),
          }),
        }
      }),
    }
  }

  fn serialize<T: serde::Serialize>(from: &T) -> Result<String> {
    serde_json::to_string(from).map_err(|e| Error::Serialize { serializer: e.to_string() })
  }

  fn deserialize<'b, T>(from: &'b Self::Intermediate) -> Result<T>
  where
    T: for<'de> serde::Deserialize<'de>,
  {
    serde_json::from_str(&from).map_err(|e| Error::Deserialize { deserializer: e.to_string() })
  }
}
