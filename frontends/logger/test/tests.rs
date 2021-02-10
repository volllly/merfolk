use merfolk::*;

#[test]
fn logger_in_process() {
  use tokio::sync::mpsc::{channel, Receiver, Sender};

  let (to, from): (Sender<merfolk_backend_in_process::InProcessChannel>, Receiver<merfolk_backend_in_process::InProcessChannel>) = channel(1);
  let logger_receiver = merfolk_frontend_logger::Logger::builder()
    .sink(|level: log::Level, string: String| println!("[{}]: {}", level, string))
    .build()
    .unwrap();

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

  log::error!("test1");
  log::warn!("test2");
  log::info!("test3");
  log::debug!("test4");
  log::trace!("test5");
}
