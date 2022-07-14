use criterion::{criterion_group, criterion_main, Criterion};
use merfolk::*;

pub fn frontend_logger(c: &mut Criterion) {
  use tokio::sync::mpsc::{channel, Receiver, Sender};

  let (to, from): (Sender<merfolk_backend_in_process::InProcessChannel>, Receiver<merfolk_backend_in_process::InProcessChannel>) = channel(1);
  let logger_receiver = merfolk_frontend_logger::Logger::builder().sink(|_: log::Level, _: String| ()).build().unwrap();

  let _merfolk_receiver = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().from(from).build().unwrap())
    .frontend(logger_receiver)
    .build()
    .unwrap();

  let logger_caller = merfolk_frontend_logger::Logger::builder()
    .level(log::Level::Trace)
    .ignore_targets(vec!["merfolk_backend_in_process"])
    .build()
    .unwrap();

  let _merfolk_caller = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().to(to).build().unwrap())
    .frontend(logger_caller)
    .build()
    .unwrap();

  c.bench_function("frontend_logger", |b| {
    b.iter(|| {
      log::trace!("bench");
    })
  });
}

criterion_group!(benches, frontend_logger);

criterion_main!(benches);
