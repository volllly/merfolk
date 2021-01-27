use mer::*;

fn add(a: i32, b: i32) -> i32 {
  a + b
}

fn sub(a: i32, b: i32) -> i32 {
  a - b
}

#[test]
fn duplex_in_process_duplex() {
  use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
  };

  let register_first_caller = mer_frontend_register::RegisterInit {}.init();
  let register_first_receiver = mer_frontend_register::RegisterInit {}.init();

  register_first_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_first_receiver.register("sub", |(a, b)| sub(a, b)).unwrap();

  let register_second_caller = mer_frontend_register::RegisterInit {}.init();
  let register_second_receiver = mer_frontend_register::RegisterInit {}.init();

  register_second_caller.register("sub", |(a, b)| sub(a, b)).unwrap();
  register_second_receiver.register("add", |(a, b)| add(a, b)).unwrap();

  let (to_first, from_first): (Sender<mer_backend_in_process::InProcessChannel>, Receiver<mer_backend_in_process::InProcessChannel>) = mpsc::channel(1);
  let (to_second, from_second): (Sender<mer_backend_in_process::InProcessChannel>, Receiver<mer_backend_in_process::InProcessChannel>) = mpsc::channel(1);

  let mut mer_first = MerInit {
    backend: mer_backend_in_process::InProcessInit {
      to: to_first.into(),
      from: from_second.into(),
    }
    .init()
    .unwrap(),
    frontend: mer_frontend_duplex::DuplexInit {
      caller: register_first_caller,
      receiver: register_first_receiver,
    }
    .init(),
    middlewares: None,
  }
  .init()
  .unwrap();

  let mut mer_second = MerInit {
    backend: mer_backend_in_process::InProcessInit {
      to: to_second.into(),
      from: from_first.into(),
    }
    .init()
    .unwrap(),
    frontend: mer_frontend_duplex::DuplexInit {
      caller: register_second_caller,
      receiver: register_second_receiver,
    }
    .init(),
    middlewares: None,
  }
  .init()
  .unwrap();

  mer_first.start().unwrap();
  mer_second.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result_first: i32 = mer_first.frontend(|f| f.caller.call("add", &(a, b)).unwrap()).unwrap();
  let result_second: i32 = mer_second.frontend(|f| f.caller.call("sub", &(a, b)).unwrap()).unwrap();

  assert_eq!(result_first, a + b);
  assert_eq!(result_second, a - b);
}
