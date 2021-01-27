use mer::*;

fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[test]
#[cfg(unix)]
#[ignore]
fn register_serialport() {
  flexi_logger::Logger::with_str("debug").start().unwrap();

  let register_caller = mer_frontend_register::RegisterInit {}.init();
  let register_receiver = mer_frontend_register::RegisterInit {}.init();
  register_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_receiver.register("add", |(a, b)| add(a, b)).unwrap();

  #[cfg(unix)]
  let pair = serialport::TTYPort::pair().unwrap();

  let mut mer_caller = MerInit {
    backend: mer_backend_serialport::SerialPortInit {
      port: Box::new(pair.0),
      poll_intervall: None,
    }
    .init()
    .unwrap(),
    frontend: register_caller,
    middlewares: None,
  }
  .init();

  let mut mer_receiver = MerInit {
    backend: mer_backend_serialport::SerialPortInit {
      port: Box::new(pair.1),
      poll_intervall: None,
    }
    .init()
    .unwrap(),
    frontend: register_receiver,
    middlewares: None,
  }
  .init();

  mer_receiver.start().unwrap();
  mer_caller.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result: i32 = mer_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result, a + b);
}
