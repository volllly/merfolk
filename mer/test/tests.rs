use mer::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn setup_empty<'a>() -> Mer<'a, backends::Empty, frontends::Empty> {
  MerInit {
    backend: backends::EmptyInit {}.init(),
    frontend: frontends::EmptyInit {}.init(),
  }
  .init()
}

fn setup_http<'a>(port: u16) -> Mer<'a, backends::Http, frontends::Empty> {
  MerInit {
    backend: backends::HttpInit {
      speak: ("http://localhost:".to_string() + &port.to_string()).parse::<hyper::Uri>().unwrap().into(),
      listen: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port).into(),
      ..Default::default()
    }
    .init(),
    frontend: frontends::EmptyInit {}.init(),
  }
  .init()
}

#[test]
fn initializes() {
  setup_empty();
  setup_http(8080);
}

#[test]
#[cfg(feature = "backends")]
fn empty_call() {
  use interfaces::Backend;
  let mut empty = backends::EmptyInit {}.init();

  empty
    .call(&Call {
      procedure: "".to_string(),
      payload: &<backends::Empty as interfaces::Backend>::serialize(&()).unwrap(),
    })
    .unwrap();
}

fn add(a: i32, b: i32) -> i32 {
  a + b
}

// impl<'a, B: interfaces::Backend<'a>> frontends::register::Call<'a, B> {
//   fn add(&self, a: i32, b: i32) -> Result<i32, B::Error> {
//     Ok(B::deserialize(
//       &(self.call)(&Call {
//         procedure: "add".to_string(),
//         payload: &B::serialize(&(a, b)).unwrap(),
//       })?
//       .payload,
//     )?)
//   }
// }

#[test]
fn frontend_call() {
  let mer = setup_empty();

  mer.call();
}

#[test]
fn frontend_http() {
  let mer = MerInit {
    backend: backends::HttpInit {
      speak: "http://volllly.free.beeceptor.com".parse::<hyper::Uri>().unwrap().into(),
      ..Default::default()
    }
    .init(),
    frontend: frontends::EmptyInit {}.init(),
  }
  .init();

  println!("1 + 2 = {:?}", mer.call());
}

#[test]
fn backend_receive() {
  let mut mer = setup_http(8081);

  mer.start().unwrap();

  // loop {
  //   std::thread::sleep(std::time::Duration::from_millis(100));
  // }
  println!("1 + 2 = {:?}", mer.call());
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
    .init(),
    frontend: register,
  }
  .init();

  mer.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let tmp = mer.call();
  assert_eq!(mer.frontend(|f| f.call("add", (a, b))), a + b);
}

#[test]
fn register_in_process() {
  let register = frontends::RegisterInit {}.init();
  register.register("add", |(a, b)| add(a, b)).unwrap();

  let mut mer = MerInit {
    backend: backends::InProcessInit {}.init(),
    frontend: register,
  }
  .init();

  mer.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  assert_eq!(mer.frontend(|f| f.call("add", (a, b))), a + b);
}

#[test]
fn derive_http() {
  #[mer_derive::frontend()]
  struct Data<T> where T: std::ops::Add<Output = T> + for<'de> serde::Deserialize<'de> + serde::Serialize + Copy + Send {
    pub offset: T
  }
  
  #[mer_derive::frontend(target = "Data")]
  trait Receiver<T> where T: std::ops::Add<Output = T> + for<'de> serde::Deserialize<'de> + serde::Serialize + Copy + Send {
    fn add(a: T, b: T) -> T::Output {
      a + b
    }
  
    fn add_with_offset(&self, a: T, b: T) -> T::Output {
      a + b + self.offset
    }
  }
  
  let mut mer = MerInit {
    backend: backends::HttpInit {
      speak: "http://localhost:8084".parse::<hyper::Uri>().unwrap().into(),
      listen: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8084).into(),
      ..Default::default()
    }
    .init(),
    frontend: DataInit::<i32> { offset: 32 }.init()
  }
  .init();

  mer.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  assert_eq!(mer.call().add_with_offset(a, b).unwrap(), a + b);
}
