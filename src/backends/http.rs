use crate::interfaces::backend;

use hyper::{Body, Client, Method, Request, StatusCode, client::{
    HttpConnector,
    connect::dns::GaiResolver
  }};

use hyper::http::Uri;

use tokio::runtime::Runtime;


pub struct Http {
  client: Client<HttpConnector<GaiResolver>, Body>,
  speak: Option<Uri>,
}

impl Default for Http {
  fn default() -> Self {
    Http {
      client: Client::new(),
      speak: None,
    }
  }
}

impl Http {
  pub fn new(speak: Option<Uri>) -> Http {
    Http {
      speak,
      ..Http::default()
    }
  }
}

impl<'a> backend::Backend<'a> for Http {
  type Intermediate = String;

  fn start(&mut self) -> Result<(), backend::Error> {
    Ok(())
  }

  fn stop(&mut self) -> Result<(), backend::Error> {
    Ok(())
  }

  // fn receiver<T>(&mut self, receiver: T) -> Result<(), backend::Error>
  // where
  //   T: Fn(&crate::Call<Self::Intermediate>) -> crate::Reply<Self::Intermediate>,
  //   T: 'a,
  //   Self::Intermediate: 'a,
  // {
  //   self.receiver = Some(Box::new(receiver));
  //   Ok(())
  // }

  #[allow(unused_variables)]
  fn call(&mut self, call: &crate::Call<Box<dyn erased_serde::Serialize>>) -> Result<crate::Reply<Self::Intermediate>, backend::Error> {
    match &self.speak {
      None => Err(backend::Error::Speak(Some(String::from("Speaking is disabled.")))),
      Some(uri) => {
        Runtime::new().unwrap().block_on(async {
          let request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Procedure", call.procedure)
            .body(Body::from(self.serialize(&call.payload)?)).map_err(|e| {backend::Error::Call(Some(e.to_string()))})?;

          let result = self.client.request(request).await;

          let response = result.map_err(|e| {backend::Error::Call(Some(e.to_string()))})?;
          let status = response.status();
          let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
          let body = String::from_utf8(body_bytes.to_vec());

          match status {
            StatusCode::OK => Ok(crate::Reply { payload: body.unwrap() }),
            _ => Err(backend::Error::Call(Some(String::from("Call returned with statuscode: ") + status.as_str())))
          }
        })
      }
    }
  }

  fn serialize(&self, from: &dyn erased_serde::Serialize) -> Result<String, backend::Error> {
    serde_json::to_string(from).map_err(|e| backend::Error::Serialize(Some(e.to_string())))
  }

  fn deserialize<'b, T>(&self, from: &'b Self::Intermediate) -> Result<T, backend::Error> where T: for<'de> serde::Deserialize<'de> {
    serde_json::from_str(&from).map_err(|e| backend::Error::Deserialize(Some(e.to_string() + "; from: " + from)))
  }
}
