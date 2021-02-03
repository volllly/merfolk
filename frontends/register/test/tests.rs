use mer::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[test]
fn register_in_process() {
  use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
  };

  let register_caller = mer_frontend_register::Register::builder().build().unwrap();
  let register_receiver = mer_frontend_register::Register::builder()
    .procedures(vec![("add", |(a, b)| add(a, b))].into_iter().collect())
    .build()
    .unwrap();
  register_caller.register("add", |(a, b)| add(a, b)).unwrap();

  let (to, from): (Sender<mer_backend_in_process::InProcessChannel>, Receiver<mer_backend_in_process::InProcessChannel>) = mpsc::channel(1);

  let mer_caller = Mer::builder()
    .backend(mer_backend_in_process::InProcess::builder().to(to).build().unwrap())
    .frontend(register_caller)
    .build()
    .unwrap();

  let mut mer_receiver = Mer::builder()
    .backend(mer_backend_in_process::InProcess::builder().from(from).build().unwrap())
    .frontend(register_receiver)
    .build()
    .unwrap();

  mer_receiver.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result: i32 = mer_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result, a + b);
}

#[test]
fn register_http() {
  let register_caller = mer_frontend_register::Register::builder().build().unwrap();
  let register_receiver = mer_frontend_register::Register::builder().build().unwrap();
  register_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_receiver.register("add", |(a, b)| add(a, b)).unwrap();

  let mer_caller = Mer::builder()
    .backend(mer_backend_http::Http::builder().speak("http://localhost:8082".parse::<hyper::Uri>().unwrap()).build().unwrap())
    .frontend(register_caller)
    .build()
    .unwrap();

  let mut mer_receiver = Mer::builder()
    .backend(
      mer_backend_http::Http::builder()
        .listen(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8082))
        .build()
        .unwrap(),
    )
    .frontend(register_receiver)
    .build()
    .unwrap();

  mer_receiver.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result: i32 = mer_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result, a + b);
}
