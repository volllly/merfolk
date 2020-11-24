use crate::interfaces::backend;

use hyper::{
  client::{connect::dns::GaiResolver, HttpConnector},
  Body, Client, Method, Request, StatusCode,
};

use hyper::http::Uri;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Response, Server};
use std::{net::SocketAddr};
use std::sync::{Arc};
use tokio::runtime::Runtime;
use tokio::sync;
use tokio::sync::{mpsc, oneshot, Mutex};

pub struct Http<'a> {
  client: Client<HttpConnector<GaiResolver>, Body>,
  speak: Option<Uri>,
  listen: Option<SocketAddr>,
  #[allow(clippy::type_complexity)]
  receiver: Option<Box<dyn Fn(Arc<crate::Call<&String>>) -> Arc<Result<crate::Reply<String>, crate::Error>> + 'a>>,
  runtime: Runtime,
  channel: (mpsc::Sender<(crate::Call<String>, oneshot::Sender<crate::Reply<String>>)>, Arc<Mutex<mpsc::Receiver<(crate::Call<String>, oneshot::Sender<crate::Reply<String>>)>>>),
}

impl<'a> Default for Http<'a> {
  fn default() -> Self {
    let channel = mpsc::channel(1);

    Http {
      client: Client::new(),
      speak: None,
      listen: None,
      receiver: None,
      runtime: Runtime::new().unwrap(),
      channel: (channel.0, Arc::new(sync::Mutex::new(channel.1)))
    }
  }
}

impl<'a> Http<'a> {
  pub fn new(speak: Option<Uri>, listen: Option<SocketAddr>) -> Http<'a> {
    Http { speak, listen, ..Http::default() }
  }
}

// async fn handle(request: Request<Body>) -> Result<Response<Body>, Infallible> {

// }

impl<'a> backend::Backend<'a> for Http<'a> {
  type Intermediate = String;

  fn start(&mut self) -> Result<(), backend::Error> {
    let listen = self.listen.unwrap();
    let tx = self.channel.0.clone();

    self.runtime.spawn(async move {
      let tx = tx.clone();

      Server::bind(&listen).serve(make_service_fn(move |_| {
      let tx = tx.clone();
      async move {
          Ok::<_, hyper::Error>(service_fn(move |request: Request<Body>| {
            let tx = tx.clone();
            async move {
              let procedure = if let Some(procedure) = request.headers().get("Procedure") {
                match procedure.to_str() {
                  Ok(procedure) => procedure.to_owned(),
                  Err(e) => return Ok::<_, hyper::Error>(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap())
                }
              } else {
                return Ok::<_, hyper::Error>(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from("No Procedure provided")).unwrap());
              };

              let body_bytes = hyper::body::to_bytes(request.into_body()).await.unwrap();
              let body = match String::from_utf8(body_bytes.to_vec()) {
                Ok(body) => body,
                Err(e) => return Ok::<_, hyper::Error>(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap()),
              };

              let (handle, rx) = oneshot::channel::<crate::Reply<String>>();

              tx.send((crate::Call {
                procedure,
                payload: body,
              }, handle)).await;

              let reply = rx.await;
              
              match reply {
                Err(e) => Ok::<_, hyper::Error>(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(e.to_string())).unwrap()),

                Ok(reply) => Ok::<_, hyper::Error>(Response::builder().status(StatusCode::OK).body(Body::from(reply.payload)).unwrap())
              }
            }
          }))
        }
      })).await.unwrap();
    });
    Ok(())
  }

  fn stop(&mut self) -> Result<(), backend::Error> {
    Ok(())
  }

  fn receiver<T>(&mut self, receiver: T) -> Result<(), backend::Error>
  where
    T: Fn(&crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>, crate::Error>,
    T: 'a,
  {
    self.receiver = Some(Box::new(move |call: Arc<crate::Call<&String>>| { 
      match receiver(call.as_ref()) {
        Ok(reply) => Arc::new(Ok(crate::Reply {
          payload: reply.payload
        })),
        Err(e) => Arc::new(Err::<crate::Reply<String>, crate::Error>(e))
      }
    }));
    
    // let rx = Arc::clone(&self.channel.1);
    // let future = async move {
    //   loop {
    //     let mut rx_mutex = rx.lock().await;
    //     let (call, tx) = rx_mutex.recv().await.unwrap();
    //     let reply = receiver(&crate::Call {procedure: call.procedure, payload: &call.payload}).unwrap();
    //     tx.send(crate::Reply { payload: reply.payload });
    //   }
    // };

    Ok(())
  }

  #[allow(unused_variables)]
  fn call(&mut self, call: &crate::Call<Self::Intermediate>) -> Result<crate::Reply<Self::Intermediate>, backend::Error> {
    match &self.speak {
      None => Err(backend::Error::Speak(Some(String::from("Speaking is disabled.")))),
      Some(uri) => self.runtime.block_on(async {
        let request = Request::builder()
          .method(Method::POST)
          .uri(uri)
          .header("Procedure", &call.procedure)
          .body(Body::from(Self::serialize(&call.payload)?))
          .map_err(|e| backend::Error::Call(Some(e.to_string())))?;

        let result = self.client.request(request).await;

        let response = result.map_err(|e| backend::Error::Call(Some(e.to_string())))?;
        let status = response.status();
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body = String::from_utf8(body_bytes.to_vec());

        match status {
          StatusCode::OK => Ok(crate::Reply { payload: body.unwrap() }),
          _ => Err(backend::Error::Call(Some(String::from("Call returned with statuscode: ") + status.as_str()))),
        }
      }),
    }
  }

  fn serialize<T : serde::Serialize>(from: &T) -> Result<String, backend::Error> {
    serde_json::to_string(from).map_err(|e| backend::Error::Serialize(Some(e.to_string())))
  }

  fn deserialize<'b, T>(from: &'b Self::Intermediate) -> Result<T, backend::Error>
  where
    T: for<'de> serde::Deserialize<'de>,
  {
    serde_json::from_str(&from).map_err(|e| backend::Error::Deserialize(Some(e.to_string() + "; from: " + from)))
  }
}

