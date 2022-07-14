use criterion::{criterion_group, criterion_main, Criterion};
use merfolk::*;
use merfolk_backend_in_process::InProcess;
use merfolk_frontend_register::Register;

pub fn frontend_register(c: &mut Criterion) {
  use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
  };

  let register_caller = merfolk_frontend_register::Register::builder().build().unwrap();
  let register_receiver = merfolk_frontend_register::Register::builder()
    .procedures(vec![("bench", Register::<InProcess>::make_procedure(|()| ()))].into_iter().collect())
    .build()
    .unwrap();

  let (to, from): (Sender<merfolk_backend_in_process::InProcessChannel>, Receiver<merfolk_backend_in_process::InProcessChannel>) = mpsc::channel(1);

  let merfolk_caller = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().to(to).build().unwrap())
    .frontend(register_caller)
    .build()
    .unwrap();

  let _merfolk_receiver = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().from(from).build().unwrap())
    .frontend(register_receiver)
    .build()
    .unwrap();

  c.bench_function("frontend_register", |b| {
    b.iter(|| {
      merfolk_caller.frontend::<_, ()>(|f| f.call("bench", &()).unwrap()).unwrap();
    })
  });
}

criterion_group!(benches, frontend_register);

criterion_main!(benches);
