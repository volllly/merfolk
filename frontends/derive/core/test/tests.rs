use mer::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[test]
fn derive_in_process() {
  #[mer_frontend_derive::frontend()]
  struct Data<T>
  where
    T: std::ops::Add<Output = T> + for<'de> serde::Deserialize<'de> + serde::Serialize + Copy + Send,
  {
    pub offset: T,
  }

  #[mer_frontend_derive::frontend(target = "Data")]
  trait Receiver<T>
  where
    T: std::ops::Add<Output = T> + for<'de> serde::Deserialize<'de> + serde::Serialize + Copy + Send,
  {
    fn add(a: T, b: T) -> T::Output {
      a + b
    }

    fn add_with_offset(&self, a: T, b: T) -> T::Output {
      a + b + self.offset
    }
  }

  let (to, from): (
    tokio::sync::mpsc::Sender<mer_backend_in_process::InProcessChannel>,
    tokio::sync::mpsc::Receiver<mer_backend_in_process::InProcessChannel>,
  ) = tokio::sync::mpsc::channel(1);

  let mer_caller = Mer::builder()
    .backend(mer_backend_in_process::InProcess::builder().to(to).build().unwrap())
    .frontend(Data::builder().offset(32).build().unwrap())
    .build()
    .unwrap();

  let _mer_register = Mer::builder()
    .backend(mer_backend_in_process::InProcess::builder().from(from).build().unwrap())
    .frontend(Data::builder().offset(32).build().unwrap())
    .build()
    .unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  assert_eq!(mer_caller.frontend(|f| { f.add(a, b).unwrap() }).unwrap(), a + b);
}

#[test]
fn derive_in_process_definition_only() {
  #[mer_frontend_derive::frontend()]
  struct DataC {}

  #[mer_frontend_derive::frontend()]

  struct DataR {}

  #[mer_frontend_derive::frontend(target = "DataC")]
  trait Caller {
    #[mer_frontend_derive::frontend(definition_only)]
    fn def();
  }

  #[mer_frontend_derive::frontend(target = "DataR")]
  trait Receiver {
    fn def() {}
  }

  let (to, from): (
    tokio::sync::mpsc::Sender<mer_backend_in_process::InProcessChannel>,
    tokio::sync::mpsc::Receiver<mer_backend_in_process::InProcessChannel>,
  ) = tokio::sync::mpsc::channel(1);

  let mer_caller = Mer::builder()
    .backend(mer_backend_in_process::InProcess::builder().to(to).build().unwrap())
    .frontend(DataC::builder().build().unwrap())
    .build()
    .unwrap();

  let _mer_register = Mer::builder()
    .backend(mer_backend_in_process::InProcess::builder().from(from).build().unwrap())
    .frontend(DataR::builder().build().unwrap())
    .build()
    .unwrap();

  mer_caller.frontend(|f| f.def().unwrap()).unwrap();
}
#[test]
fn derive_http() {
  #[mer_frontend_derive::frontend()]
  struct Data<T>
  where
    T: std::ops::Add<Output = T> + for<'de> serde::Deserialize<'de> + serde::Serialize + Copy + Send,
  {
    pub offset: T,
  }

  #[mer_frontend_derive::frontend(target = "Data")]
  trait Receiver<T>
  where
    T: std::ops::Add<Output = T> + for<'de> serde::Deserialize<'de> + serde::Serialize + Copy + Send,
  {
    fn add(a: T, b: T) -> T::Output {
      a + b
    }

    fn add_with_offset(&self, a: T, b: T) -> T::Output {
      a + b + self.offset
    }
  }

  let mer_caller = Mer::builder()
    .backend(mer_backend_http::Http::builder().speak("http://localhost:8083".parse::<hyper::Uri>().unwrap()).build().unwrap())
    .frontend(Data::builder().offset(32).build().unwrap())
    .build()
    .unwrap();

  let _mer_register = Mer::builder()
    .backend(
      mer_backend_http::Http::builder()
        .listen(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8083))
        .build()
        .unwrap(),
    )
    .frontend(Data::builder().offset(32).build().unwrap())
    .build()
    .unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  assert_eq!(mer_caller.frontend(|f| { f.add(a, b).unwrap() }).unwrap(), a + b);
}
