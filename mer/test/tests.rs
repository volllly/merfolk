use backends::{InProcess, InProcessChannel};
use flexi_logger::Logger;
use mer::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[test]
fn register_http() {
  let register = frontends::RegisterInit {}.init();
  register.register("add", |(a, b)| add(a, b)).unwrap();

  let mut mer = MerInit {
    backend: backends::HttpInit {
      speak: "http://localhost:8084".parse::<hyper::Uri>().unwrap().into(),
      listen: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8084).into(),
      ..Default::default()
    }
    .init().unwrap(),
    frontend: register,
  }
  .init();

  mer.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result: i32 = mer.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result, a + b);
}

#[test]
fn register_in_process() {
  use std::sync::mpsc;
  use std::sync::mpsc::{Sender, Receiver};

  let register_caller = frontends::RegisterInit {}.init();
  let register_receiver = frontends::RegisterInit {}.init();
  register_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_receiver.register("add", |(a, b)| add(a, b)).unwrap();

  let (to, from): (Sender<InProcessChannel>, Receiver<InProcessChannel>) = mpsc::channel();
  
  let mut mer_caller = MerInit {
    backend: backends::InProcessInit { to: to.into(), ..Default::default() }.init().unwrap(),
    frontend: register_caller,
  }
  .init();

  let mut mer_receiver = MerInit {
    backend: backends::InProcessInit { from: from.into(), ..Default::default() }.init().unwrap(),
    frontend: register_receiver,
  }
  .init();

  mer_receiver.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result: i32 = mer_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result, a + b);
}

#[test]
fn derive_http() {
  Logger::with_str("trace").start().unwrap();

  #[mer_derive::frontend()]
  struct Data<T>
  where
    T: std::ops::Add<Output = T> + for<'de> serde::Deserialize<'de> + serde::Serialize + Copy + Send,
  {
    pub offset: T,
  }

  #[mer_derive::frontend(target = "Data")]
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

  let mer_call = MerInit {
    backend: backends::HttpInit {
      speak: "http://localhost:8084".parse::<hyper::Uri>().unwrap().into(),
      listen: None,
      ..Default::default()
    }
    .init().unwrap(),
    frontend: DataInit::<i32> { offset: 32 }.init(),
  }
  .init();

  let mut mer_receive = MerInit {
    backend: backends::HttpInit {
      speak: None,
      listen: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8084).into(),
      ..Default::default()
    }
    .init().unwrap(),
    frontend: DataInit::<i32> { offset: 32 }.init(),
  }
  .init();

  mer_receive.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  assert_eq!(
    mer_call
      .frontend(|f| {
        println!("start");

        let tmp = f.add(a, b).unwrap();
        tmp
      })
      .unwrap(),
    a + b
  );
}

#[test]
fn derive_in_process() {
  #[mer_derive::frontend()]
  struct Data<T>
  where
    T: std::ops::Add<Output = T> + for<'de> serde::Deserialize<'de> + serde::Serialize + Copy + Send,
  {
    pub offset: T,
  }

  #[mer_derive::frontend(target = "Data")]
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

  let mut mer = MerInit {
    backend: backends::InProcessInit { ..Default::default() }.init().unwrap(),
    frontend: DataInit::<i32> { offset: 32 }.init(),
  }
  .init();

  mer.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  assert_eq!(mer.frontend(|f| { f.add(a, b).unwrap() }).unwrap(), a + b);
}
