use mer::*;

fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[test]
fn register_in_process() {
  use tokio::sync::mpsc::{channel, Receiver, Sender};

  let register_caller = mer_frontend_register::Register::builder().build().unwrap();
  let register_receiver = mer_frontend_register::Register::builder().build().unwrap();
  register_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_receiver.register("add", |(a, b)| add(a, b)).unwrap();

  let (to, from): (Sender<mer_backend_in_process::InProcessChannel>, Receiver<mer_backend_in_process::InProcessChannel>) = channel(1);

  let mer_caller = Mer::builder()
    .backend(mer_backend_in_process::InProcess::builder().to(to).build().unwrap())
    .frontend(register_caller)
    .build()
    .unwrap();

  let _mer_receiver = Mer::builder()
    .backend(mer_backend_in_process::InProcess::builder().from(from).build().unwrap())
    .frontend(register_receiver)
    .build()
    .unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result: i32 = mer_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result, a + b);
}

#[test]
fn register_in_process_duplex() {
  use tokio::sync::mpsc::{channel, Receiver, Sender};

  let register_first = mer_frontend_register::Register::builder().build().unwrap();
  let register_second = mer_frontend_register::Register::builder().build().unwrap();

  register_first.register("add", |(a, b)| add(a, b)).unwrap();
  register_second.register("add", |(a, b)| add(a, b)).unwrap();

  let (to_first, from_first): (Sender<mer_backend_in_process::InProcessChannel>, Receiver<mer_backend_in_process::InProcessChannel>) = channel(1);
  let (to_second, from_second): (Sender<mer_backend_in_process::InProcessChannel>, Receiver<mer_backend_in_process::InProcessChannel>) = channel(1);

  let mer_first = Mer::builder()
    .backend(mer_backend_in_process::InProcess::builder().to(to_first).from(from_second).build().unwrap())
    .frontend(register_first)
    .build()
    .unwrap();

  let mer_second = Mer::builder()
    .backend(mer_backend_in_process::InProcess::builder().to(to_second).from(from_first).build().unwrap())
    .frontend(register_second)
    .build()
    .unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result_first: i32 = mer_first.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result_first, a + b);

  let (x, y) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result_second: i32 = mer_second.frontend(|f| f.call("add", &(x, y)).unwrap()).unwrap();
  assert_eq!(result_second, x + y);
}
