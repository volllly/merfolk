use mer::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[test]
fn register_http() {
  let register_caller = mer_frontend_register::RegisterInit {}.init();
  let register_receiver = mer_frontend_register::RegisterInit {}.init();
  register_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_receiver.register("add", |(a, b)| add(a, b)).unwrap();

  let mer_caller = MerInit {
    backend: mer_backend_http::HttpInit {
      speak: "http://localhost:8080".parse::<hyper::Uri>().unwrap().into(),
      ..Default::default()
    }
    .init()
    .unwrap(),
    frontend: register_caller,
  }
  .init();

  let mut mer_receiver = MerInit {
    backend: mer_backend_http::HttpInit {
      listen: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080).into(),
      ..Default::default()
    }
    .init()
    .unwrap(),
    frontend: register_receiver,
  }
  .init();

  mer_receiver.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result: i32 = mer_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result, a + b);
}
