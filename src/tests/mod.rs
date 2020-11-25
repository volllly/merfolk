use super::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn setup_empty<'a>() -> Mer<'a, backends::Empty, frontends::Empty> {
  Mer::new().with_backend(backends::Empty::new()).with_frontnd(frontends::Empty::new()).build()
}

fn setup_http<'a>(port: u16) -> Mer<'a, backends::Http, frontends::Empty> {
  Mer::new()
    .with_backend(backends::Http::new(
      Some("http://127.0.0.1:8080".parse::<hyper::Uri>().unwrap()),
      Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)),
    ))
    .with_frontnd(frontends::Empty::new())
    .build()
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
  let mut empty = backends::Empty::new();

  empty
    .call(&Call {
      procedure: "".to_string(),
      payload: <backends::Empty as interfaces::Backend>::serialize(&()).unwrap(),
    })
    .unwrap();
}

#[allow(dead_code)]
fn add(a: i32, b: i32) -> i32 {
  a + b
}

impl<'a> Caller<'a, backends::Http> {
  fn add(&self, a: i32, b: i32) -> Result<i32, interfaces::backend::Error> {
    Ok(
      self
        .call(&Call {
          procedure: "add".to_string(),
          payload: <backends::Http as interfaces::Backend>::serialize(&(a, b)).unwrap(),
        })?
        .payload,
    )
  }
}

impl<'a> Caller<'a, backends::Empty> {
  fn add(&self, a: i32, b: i32) -> Result<i32, interfaces::backend::Error> {
    Ok(
      self
        .call(&Call {
          procedure: "add".to_string(),
          payload: <backends::Empty as interfaces::Backend>::serialize(&(a, b)).unwrap(),
        })?
        .payload,
    )
  }
}

#[test]
fn frontend_call() {
  let mer = setup_empty();

  assert_eq!(mer.call.add(1, 2).is_ok(), false);
}

#[test]
fn frontend_http() {
  let mer = Mer::new()
    .with_backend(backends::Http::new(Some("http://volllly.free.beeceptor.com".parse::<hyper::Uri>().unwrap()), None))
    .with_frontnd(frontends::Empty::new())
    .build();

  println!("1 + 2 = {}", mer.call.add(1, 2).unwrap());
}

#[test]
fn frontend_receive() {
  let mer = setup_empty();

  mer
    .receive(&Call {
      procedure: "add".to_string(),
      payload: &serde_json::to_string(&(1i32, 2i32)).unwrap(),
    })
    .unwrap();
}

#[test]
fn backend_receive() {
  let mut mer = setup_http(8081);

  mer.start().unwrap();

  // loop {
  //   std::thread::sleep(std::time::Duration::from_millis(100));
  // }
  println!("1 + 2 = {}", mer.call.add(1, 2).unwrap());
}
