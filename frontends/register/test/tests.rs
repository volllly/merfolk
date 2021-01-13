use mer::*;

fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[test]
fn register_in_process() {
  use tokio::sync::mpsc;
  use tokio::sync::mpsc::{Receiver, Sender};

  let register_caller = mer_frontend_register::RegisterInit {}.init();
  let register_receiver = mer_frontend_register::RegisterInit {}.init();
  register_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_receiver.register("add", |(a, b)| add(a, b)).unwrap();

  let (to, from): (Sender<mer_backend_in_process::InProcessChannel>, Receiver<mer_backend_in_process::InProcessChannel>) = mpsc::channel(1);

  let mer_caller = MerInit {
    backend: mer_backend_in_process::InProcessInit { to: to.into(), ..Default::default() }.init().unwrap(),
    frontend: register_caller,
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
  }
  .init();

  mer_receiver.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result: i32 = mer_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result, a + b);
}
