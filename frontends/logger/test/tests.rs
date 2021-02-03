use mer::*;

#[test]
fn logger_in_process() {
  use tokio::sync::mpsc::{channel, Receiver, Sender};

  let (to, from): (Sender<mer_backend_in_process::InProcessChannel>, Receiver<mer_backend_in_process::InProcessChannel>) = channel(1);
  let logger_receiver = mer_frontend_logger::Logger::builder()
    .sink(|level: log::Level, string: String| println!("[{}]: {}", level, string))
    .build()
    .unwrap();

  let mut mer_receiver = Mer::builder()
    .backend(mer_backend_in_process::InProcess::builder().from(from.into()).build().unwrap())
    .frontend(logger_receiver)
    .build()
    .unwrap();

  mer_receiver.start().unwrap();

  let logger_caller = mer_frontend_logger::Logger::builder()
    .level(log::Level::Trace)
    .ignore_targets(vec!["mer_backend_in_process"])
    .build()
    .unwrap();

  let _mer_caller = Mer::builder()
    .backend(mer_backend_in_process::InProcess::builder().to(to).build().unwrap())
    .frontend(logger_caller)
    .build()
    .unwrap();

  log::error!("test1");
  log::warn!("test2");
  log::info!("test3");
  log::debug!("test4");
  log::trace!("test5");
}
