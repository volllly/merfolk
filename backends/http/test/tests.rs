use mer::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[test]
fn register_http() {
  let register_caller = mer_frontend_register::Register::builder().build().unwrap();
  let register_receiver = mer_frontend_register::Register::builder().build().unwrap();
  register_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_receiver.register("add", |(a, b)| add(a, b)).unwrap();

  let mer_caller = Mer::builder()
    .backend(mer_backend_http::Http::builder().speak("http://localhost:8080".parse::<hyper::Uri>().unwrap()).build().unwrap())
    .frontend(register_caller)
    .build()
    .unwrap();

  let mer_receiver = Mer::builder()
    .backend(
      mer_backend_http::Http::builder()
        .listen(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080))
        .build()
        .unwrap(),
    )
    .frontend(register_receiver)
    .build()
    .unwrap();

  mer_receiver.backend(|b| b.start().unwrap()).unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result: i32 = mer_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result, a + b);
}

#[test]
fn register_http_duplex() {
  let register_first = mer_frontend_register::Register::builder().build().unwrap();
  let register_second = mer_frontend_register::Register::builder().build().unwrap();

  register_first.register("add", |(a, b)| add(a, b)).unwrap();
  register_second.register("add", |(a, b)| add(a, b)).unwrap();

  let mer_first = Mer::builder()
    .backend(
      mer_backend_http::Http::builder()
        .listen(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8085))
        .speak("http://localhost:8084".parse::<hyper::Uri>().unwrap())
        .build()
        .unwrap(),
    )
    .frontend(register_first)
    .build()
    .unwrap();

  let mer_second = Mer::builder()
    .backend(
      mer_backend_http::Http::builder()
        .speak("http://localhost:8085".parse::<hyper::Uri>().unwrap())
        .listen(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8084))
        .build()
        .unwrap(),
    )
    .frontend(register_second)
    .build()
    .unwrap();

  mer_first.backend(|b| b.start().unwrap()).unwrap();
  mer_second.backend(|b| b.start().unwrap()).unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result_first: i32 = mer_first.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result_first, a + b);

  let (x, y) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result_second: i32 = mer_second.frontend(|f| f.call("add", &(x, y)).unwrap()).unwrap();
  assert_eq!(result_second, x + y);
}
