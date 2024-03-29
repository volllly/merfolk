use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use criterion::{criterion_group, criterion_main, Criterion};
use merfolk::*;

pub fn backend_http(c: &mut Criterion) {
  let register_caller = merfolk_frontend_register::Register::builder().build().unwrap();
  let register_receiver = merfolk_frontend_register::Register::builder().build().unwrap();
  register_receiver.register("bench", |()| ()).unwrap();

  let merfolk_caller = Mer::builder()
    .backend(merfolk_backend_http::Http::builder().speak("http://localhost:8080".parse::<hyper::Uri>().unwrap()).build().unwrap())
    .frontend(register_caller)
    .build()
    .unwrap();

  let _merfolk_receiver = Mer::builder()
    .backend(
      merfolk_backend_http::Http::builder()
        .listen(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080))
        .build()
        .unwrap(),
    )
    .frontend(register_receiver)
    .build()
    .unwrap();

  c.bench_function("backend_http", |b| {
    b.iter(|| {
      merfolk_caller.frontend::<_, ()>(|f| f.call("bench", &()).unwrap()).unwrap();
    })
  });
}

criterion_group!(benches, backend_http);

criterion_main!(benches);
