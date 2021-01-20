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
    middlewares: None
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
    middlewares: None
  }
  .init();

  mer_receiver.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result: i32 = mer_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result, a + b);
}

#[test]
fn register_http_duplex() {
  let register_first = mer_frontend_register::RegisterInit {}.init();
  let register_second = mer_frontend_register::RegisterInit {}.init();

  register_first.register("add", |(a, b)| add(a, b)).unwrap();
  register_second.register("add", |(a, b)| add(a, b)).unwrap();

  let mut mer_first = MerInit {
    backend: mer_backend_http::HttpInit {
      listen: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8085).into(),
      speak: "http://localhost:8084".parse::<hyper::Uri>().unwrap().into(),
      ..Default::default()
    }
    .init()
    .unwrap(),
    frontend: register_first,
    middlewares: None
  }
  .init();

  let mut mer_second = MerInit {
    backend: mer_backend_http::HttpInit {
      speak: "http://localhost:8085".parse::<hyper::Uri>().unwrap().into(),
      listen: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8084).into(),
      ..Default::default()
    }
    .init()
    .unwrap(),
    frontend: register_second,
    middlewares: None
  }
  .init();

  mer_first.start().unwrap();
  mer_second.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result_first: i32 = mer_first.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result_first, a + b);

  let (x, y) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result_second: i32 = mer_second.frontend(|f| f.call("add", &(x, y)).unwrap()).unwrap();
  assert_eq!(result_second, x + y);
}
