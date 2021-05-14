use merfolk::*;

use criterion::{criterion_group, criterion_main, Criterion};

pub fn frontend_derive(c: &mut Criterion) {
  #[merfolk_frontend_derive::frontend()]
  struct Data {}

  #[merfolk_frontend_derive::frontend(target = "Data")]
  trait Receiver
  {
    fn bench() {}
  }

  let (to, from): (
    tokio::sync::mpsc::Sender<merfolk_backend_in_process::InProcessChannel>,
    tokio::sync::mpsc::Receiver<merfolk_backend_in_process::InProcessChannel>,
  ) = tokio::sync::mpsc::channel(1);

  let merfolk_caller = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().to(to).build().unwrap())
    .frontend(Data::builder().build().unwrap())
    .build()
    .unwrap();

  let _merfolk_register = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().from(from).build().unwrap())
    .frontend(Data::builder().build().unwrap())
    .build()
    .unwrap();

  c.bench_function("frontend_derive", |b| b.iter(|| { let _: () = merfolk_caller.frontend(|f| f.bench().unwrap()).unwrap(); }));
}

criterion_group!(benches, frontend_derive);

criterion_main!(benches);
