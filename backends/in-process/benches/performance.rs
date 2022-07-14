use criterion::{criterion_group, criterion_main, Criterion};
use merfolk::*;

pub fn backend_in_process(c: &mut Criterion) {
  use tokio::sync::mpsc::{channel, Receiver, Sender};

  let register_caller = merfolk_frontend_register::Register::builder().build().unwrap();
  let register_receiver = merfolk_frontend_register::Register::builder().build().unwrap();
  register_receiver.register("bench", |()| ()).unwrap();

  let (to, from): (Sender<merfolk_backend_in_process::InProcessChannel>, Receiver<merfolk_backend_in_process::InProcessChannel>) = channel(1);

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

  c.bench_function("backend_in_process", |b| {
    b.iter(|| {
      merfolk_caller.frontend::<_, ()>(|f| f.call("bench", &()).unwrap()).unwrap();
    })
  });
}

criterion_group!(benches, backend_in_process);

criterion_main!(benches);
