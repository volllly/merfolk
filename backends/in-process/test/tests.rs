use mer::*;

fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[test]
fn register_in_process() {
  use tokio::sync::mpsc::{channel, Receiver, Sender};

  let register_caller = mer_frontend_register::RegisterInit {}.init();
  let register_receiver = mer_frontend_register::RegisterInit {}.init();
  register_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_receiver.register("add", |(a, b)| add(a, b)).unwrap();

  let (to, from): (Sender<mer_backend_in_process::InProcessChannel>, Receiver<mer_backend_in_process::InProcessChannel>) = channel(1);

  let mer_caller = MerInit {
    backend: mer_backend_in_process::InProcessInit { to: to.into(), ..Default::default() }.init().unwrap(),
    frontend: register_caller,
    middlewares: None,
  }
  .init();

  let mut mer_receiver = MerInit {
    backend: mer_backend_in_process::InProcessInit {
      from: from.into(),
      ..Default::default()
    }
    .init()
    .unwrap(),
    frontend: register_receiver,
    middlewares: None,
  }
  .init();

  mer_receiver.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result: i32 = mer_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result, a + b);
}

#[test]
fn register_in_process_duplex() {
  use tokio::sync::mpsc::{channel, Receiver, Sender};

  let register_first = mer_frontend_register::RegisterInit {}.init();
  let register_second = mer_frontend_register::RegisterInit {}.init();

  register_first.register("add", |(a, b)| add(a, b)).unwrap();
  register_second.register("add", |(a, b)| add(a, b)).unwrap();

  let (to_first, from_first): (Sender<mer_backend_in_process::InProcessChannel>, Receiver<mer_backend_in_process::InProcessChannel>) = channel(1);
  let (to_second, from_second): (Sender<mer_backend_in_process::InProcessChannel>, Receiver<mer_backend_in_process::InProcessChannel>) = channel(1);

  let mut mer_first = MerInit {
    backend: mer_backend_in_process::InProcessInit {
      to: to_first.into(),
      from: from_second.into(),
    }
    .init()
    .unwrap(),
    frontend: register_first,
    middlewares: None,
  }
  .init();

  let mut mer_second = MerInit {
    backend: mer_backend_in_process::InProcessInit {
      to: to_second.into(),
      from: from_first.into(),
    }
    .init()
    .unwrap(),
    frontend: register_second,
    middlewares: None,
  }
  .init();

  mer_first.start().unwrap();
  mer_second.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result_first: i32 = mer_first.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result_first, a + b);

  let (x, y) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result_second: i32 = mer_second.frontend(|f| f.call("add", &(x, y)).unwrap()).unwrap();
  assert_eq!(result_second, x + y);
}
