use criterion::{criterion_group, criterion_main, Criterion};
use merfolk::*;

pub fn middleware_authentication(c: &mut Criterion) {
  use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
  };

  let register_caller = merfolk_frontend_register::Register::builder().build().unwrap();
  let register_receiver = merfolk_frontend_register::Register::builder().build().unwrap();
  register_caller.register("bench", |()| ()).unwrap();
  register_receiver.register("bench", |()| ()).unwrap();

  let (to, from): (Sender<merfolk_backend_in_process::InProcessChannel>, Receiver<merfolk_backend_in_process::InProcessChannel>) = mpsc::channel(1);

  let merfolk_caller = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().to(to).build().unwrap())
    .frontend(register_caller)
    .middlewares(vec![merfolk_middleware_authentication::Authentication::builder()
      .auth(("u".to_string(), "p".to_string()))
      .build_boxed()
      .unwrap()])
    .build()
    .unwrap();

  let _merfolk_receiver = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().from(from).build().unwrap())
    .frontend(register_receiver)
    .middlewares(vec![merfolk_middleware_authentication::Authentication::builder()
      .authenticator(move |_: (String, String), _: Vec<String>| Ok(()))
      .build_boxed()
      .unwrap()])
    .build()
    .unwrap();

  c.bench_function("middleware_authentication", |b| {
    b.iter(|| {
      merfolk_caller.frontend::<_, ()>(|f| f.call("bench", &()).unwrap()).unwrap();
    })
  });
}

criterion_group!(benches, middleware_authentication);

criterion_main!(benches);
