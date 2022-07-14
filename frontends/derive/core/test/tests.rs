use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use merfolk::*;

#[test]
fn derive_in_process() {
  #[merfolk_frontend_derive::frontend()]
  struct Data<T>
  where
    T: std::ops::Add<Output = T> + for<'de> serde::Deserialize<'de> + serde::Serialize + Copy + Send,
  {
    pub offset: T,
  }

  #[merfolk_frontend_derive::frontend(target = "Data")]
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
    tokio::sync::mpsc::Sender<merfolk_backend_in_process::InProcessChannel>,
    tokio::sync::mpsc::Receiver<merfolk_backend_in_process::InProcessChannel>,
  ) = tokio::sync::mpsc::channel(1);

  let merfolk_caller = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().to(to).build().unwrap())
    .frontend(Data::builder().offset(32).build().unwrap())
    .build()
    .unwrap();

  let _merfolk_register = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().from(from).build().unwrap())
    .frontend(Data::builder().offset(32).build().unwrap())
    .build()
    .unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  assert_eq!(merfolk_caller.frontend(|f| { f.add(a, b).unwrap() }).unwrap(), a + b);
}

#[test]
fn derive_in_process_definition_only() {
  #[merfolk_frontend_derive::frontend()]
  struct DataC {}

  #[merfolk_frontend_derive::frontend()]

  struct DataR {}

  #[merfolk_frontend_derive::frontend(target = "DataC")]
  trait Caller {
    #[merfolk_frontend_derive::frontend(definition_only)]
    fn def();
  }

  #[merfolk_frontend_derive::frontend(target = "DataR")]
  trait Receiver {
    fn def() {}
  }

  let (to, from): (
    tokio::sync::mpsc::Sender<merfolk_backend_in_process::InProcessChannel>,
    tokio::sync::mpsc::Receiver<merfolk_backend_in_process::InProcessChannel>,
  ) = tokio::sync::mpsc::channel(1);

  let merfolk_caller = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().to(to).build().unwrap())
    .frontend(DataC::builder().build().unwrap())
    .build()
    .unwrap();

  let _merfolk_register = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().from(from).build().unwrap())
    .frontend(DataR::builder().build().unwrap())
    .build()
    .unwrap();

  merfolk_caller.frontend(|f| f.def().unwrap()).unwrap();
}
#[test]
fn derive_http() {
  #[merfolk_frontend_derive::frontend()]
  struct Data<T>
  where
    T: std::ops::Add<Output = T> + for<'de> serde::Deserialize<'de> + serde::Serialize + Copy + Send,
  {
    pub offset: T,
  }

  #[merfolk_frontend_derive::frontend(target = "Data")]
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

  let merfolk_caller = Mer::builder()
    .backend(merfolk_backend_http::Http::builder().speak("http://localhost:8083".parse::<hyper::Uri>().unwrap()).build().unwrap())
    .frontend(Data::builder().offset(32).build().unwrap())
    .build()
    .unwrap();

  let _merfolk_register = Mer::builder()
    .backend(
      merfolk_backend_http::Http::builder()
        .listen(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8083))
        .build()
        .unwrap(),
    )
    .frontend(Data::builder().offset(32).build().unwrap())
    .build()
    .unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  assert_eq!(merfolk_caller.frontend(|f| { f.add(a, b).unwrap() }).unwrap(), a + b);
}
