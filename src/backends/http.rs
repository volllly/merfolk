use crate::interfaces::backend;

use hyper::{
  client::{connect::dns::GaiResolver, HttpConnector},
  Body, Client, Method, Request, StatusCode,
};

use hyper::http::Uri;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Response, Server};
use std::{net::SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

pub struct Http {
  client: Client<HttpConnector<GaiResolver>, Body>,
  speak: Option<Uri>,
  listen: Option<SocketAddr>,
  #[allow(clippy::type_complexity)]
  receiver: Option<Arc<Mutex<dyn Fn(Arc<crate::Call<&String>>) -> Arc<Result<crate::Reply<Box<dyn erased_serde::Serialize>>, crate::Error>> + 'static + Send + Sync>>>,
  runtime: Runtime
}

impl Default for Http {
  fn default() -> Self {
    Http {
      client: Client::new(),
      speak: None,
      listen: None,
      receiver: None,
      runtime: Runtime::new().unwrap()
    }
  }
}

impl Http {
  pub fn new(speak: Option<Uri>, listen: Option<SocketAddr>) -> Http {
    Http { speak, listen, ..Http::default() }
  }
}

// async fn handle(request: Request<Body>) -> Result<Response<Body>, Infallible> {

// }

impl<'a> backend::Backend<'a> for Http {
  type Intermediate = String;

  fn start(&mut self) -> Result<(), backend::Error> {
    let listen = self.listen;
    let receiver = Arc::clone(self.receiver.as_ref().unwrap());

    self.runtime.spawn(async move {
      let server = Server::bind(&listen.unwrap()).serve(make_service_fn(move |_| {
        let receiver = Arc::clone(&receiver);

        async move {
          let receiver = Arc::clone(&receiver);
          Ok::<_, hyper::Error>(service_fn(move |request: Request<Body>| {
            let receiver = Arc::clone(&receiver);
          
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

              let reply = receiver.lock().unwrap()(Arc::new(crate::Call {
                procedure: &procedure,
                payload: &body,
              }));

              match &*reply {
                Err(e) => Ok::<_, hyper::Error>(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(format!("{:?}", e))).unwrap()),

                Ok(reply) => {
                  let ser = Self::serialize(reply.payload.as_ref()).unwrap();
                  Ok::<_, hyper::Error>(Response::builder().status(StatusCode::OK).body(Body::from(ser)).unwrap())
                }
              }
            }
          }))
        }
      }));

      println!("{:?}", server);
      server.await.unwrap();
    });
    Ok(())
  }

  fn stop(&mut self) -> Result<(), backend::Error> {
    Ok(())
  }

  fn receiver<T>(&mut self, receiver: T) -> Result<(), backend::Error>
  where
    T: Fn(&crate::Call<&Self::Intermediate>) -> Result<crate::Reply<Box<dyn erased_serde::Serialize>>, crate::Error> + Send + Sync,
    T: 'static,
  {
    self.receiver = Some(Arc::new(Mutex::new(move |call: Arc<crate::Call<&String>>| { 
      match receiver(call.as_ref()) {
        Ok(reply) => Arc::new(Ok(crate::Reply {
          payload: reply.payload
        })),
        Err(e) => Arc::new(Err::<crate::Reply<Box<dyn erased_serde::Serialize>>, crate::Error>(e))
      }
    })));
    Ok(())
  }

  #[allow(unused_variables)]
  fn call(&mut self, call: &crate::Call<Box<dyn erased_serde::Serialize>>) -> Result<crate::Reply<Self::Intermediate>, backend::Error> {
    match &self.speak {
      None => Err(backend::Error::Speak(Some(String::from("Speaking is disabled.")))),
      Some(uri) => Runtime::new().unwrap().block_on(async {
        let request = Request::builder()
          .method(Method::POST)
          .uri(uri)
          .header("Procedure", call.procedure)
          .body(Body::from(Self::serialize(call.payload.as_ref())?))
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

  fn serialize<'b>(from: &'b dyn erased_serde::Serialize) -> Result<String, backend::Error> {
    serde_json::to_string(from).map_err(|e| backend::Error::Serialize(Some(e.to_string())))
  }

  fn deserialize<'b, T>(from: &'b Self::Intermediate) -> Result<T, backend::Error>
  where
    T: for<'de> serde::Deserialize<'de>,
  {
    serde_json::from_str(&from).map_err(|e| backend::Error::Deserialize(Some(e.to_string() + "; from: " + from)))
  }
}

