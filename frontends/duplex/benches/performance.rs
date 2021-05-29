use merfolk::*;

use criterion::{criterion_group, criterion_main, Criterion};

pub fn frontend_duplex(c: &mut Criterion) {
  use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
  };

  let register_first_caller = merfolk_frontend_register::Register::builder().build().unwrap();
  let register_first_receiver = merfolk_frontend_register::Register::builder().build().unwrap();

  let register_second_caller = merfolk_frontend_register::Register::builder().build().unwrap();
  let register_second_receiver = merfolk_frontend_register::Register::builder().build().unwrap();

  register_second_receiver.register("bench", |()| ()).unwrap();

  let (to_first, from_first): (Sender<merfolk_backend_in_process::InProcessChannel>, Receiver<merfolk_backend_in_process::InProcessChannel>) = mpsc::channel(1);
  let (to_second, from_second): (Sender<merfolk_backend_in_process::InProcessChannel>, Receiver<merfolk_backend_in_process::InProcessChannel>) = mpsc::channel(1);

  let merfolk_first = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().to(to_first).from(from_second).build().unwrap())
    .frontend(merfolk_frontend_duplex::Duplex::builder().caller(register_first_caller).receiver(register_first_receiver).build().unwrap())
    .build()
    .unwrap();

  let _merfolk_second = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().to(to_second).from(from_first).build().unwrap())
    .frontend(
      merfolk_frontend_duplex::Duplex::builder()
        .caller(register_second_caller)
        .receiver(register_second_receiver)
        .build()
        .unwrap(),
    )
    .build()
    .unwrap();

  c.bench_function("frontend_duplex", |b| b.iter(|| { let _: () = merfolk_first.frontend(|f| f.caller.call("bench", &()).unwrap()).unwrap(); }));
}

criterion_group!(benches, frontend_duplex);

criterion_main!(benches);
