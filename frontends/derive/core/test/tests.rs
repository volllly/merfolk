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

  let mer_caller = MerInit {
    backend: mer_backend_in_process::InProcessInit { to: to.into(), ..Default::default() }.init().unwrap(),
    frontend: DataInit::<i32> { offset: 32 }.init(),
  }
  .init();

  let mut mer_register = MerInit {
    backend: mer_backend_in_process::InProcessInit {
      from: from.into(),
      ..Default::default()
    }
    .init()
    .unwrap(),
    frontend: DataInit::<i32> { offset: 32 }.init(),
  }
  .init();

  mer_register.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  assert_eq!(mer_caller.frontend(|f| { f.add(a, b).unwrap() }).unwrap(), a + b);
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

  let mer_caller = MerInit {
    backend: mer_backend_http::HttpInit {
      speak: "http://localhost:8083".parse::<hyper::Uri>().unwrap().into(),
      ..Default::default()
    }
    .init()
    .unwrap(),
    frontend: DataInit::<i32> { offset: 32 }.init(),
  }
  .init();

  let mut mer_register = MerInit {
    backend: mer_backend_http::HttpInit {
      listen: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8083).into(),
      ..Default::default()
    }
    .init()
    .unwrap(),
    frontend: DataInit::<i32> { offset: 32 }.init(),
  }
  .init();

  mer_register.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  assert_eq!(mer_caller.frontend(|f| { f.add(a, b).unwrap() }).unwrap(), a + b);
}
