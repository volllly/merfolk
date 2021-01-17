use mer::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[test]
fn logger_caller_in_process() {
  use tokio::sync::mpsc::{channel, Receiver, Sender};
  let (to, from): (Sender<mer_backend_in_process::InProcessChannel>, Receiver<mer_backend_in_process::InProcessChannel>) = channel(1);

  let logger_receiver = mer_frontend_logger::LoggerInit {
    sink: Some(Box::new(|level: log::Level, string: String| println!("[{}]: {}", level, string))),
    ..Default::default()
  }
  .init();

  let mut mer_receiver = MerInit {
    backend: mer_backend_in_process::InProcessInit {
      from: from.into(),
      ..Default::default()
    }
    .init()
    .unwrap(),
    frontend: logger_receiver,
  }
  .init();

  mer_receiver.start().unwrap();

  let logger_caller = mer_frontend_logger::LoggerInit {
    level: log::Level::Trace.into(),
    ignore_crates: vec!["mer_backend_in_process"],
    ..Default::default()
  }
  .init();

  let _mer_caller = MerInit {
    backend: mer_backend_in_process::InProcessInit { to: to.into(), ..Default::default() }.init().unwrap(),
    frontend: logger_caller,
  }
  .init();

  log::error!("test1");
  log::warn!("test2");
  log::info!("test3");
  log::debug!("test4");
  log::trace!("test5");
}


#[test]
fn logger_caller_http() {

  let logger_receiver = mer_frontend_logger::LoggerInit {
    sink: Some(Box::new(|level: log::Level, string: String| println!("[{}]: {}", level, string))),
    ..Default::default()
  }
  .init();

  let mut mer_receiver = MerInit {
    backend: mer_backend_http::HttpInit {
      listen: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081).into(),
      ..Default::default()
    }
    .init()
    .unwrap(),
    frontend: logger_receiver,
  }
  .init();
  
  mer_receiver.start().unwrap();


  let logger_caller = mer_frontend_logger::LoggerInit {
    level: log::Level::Trace.into(),
    ignore_crates: vec!["mer_backend_http"],
    ..Default::default()
  }
  .init();


  let _mer_caller = MerInit {
    backend: mer_backend_http::HttpInit { speak: "http://localhost:8081".parse::<hyper::Uri>().unwrap().into(), ..Default::default() }.init().unwrap(),
    frontend: logger_caller,
  }
  .init();

  log::error!("test1");
}
