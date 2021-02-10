use merfolk::*;

fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[test]
fn register_in_process() {
  use tokio::sync::mpsc::{channel, Receiver, Sender};

  let register_caller = merfolk_frontend_register::Register::builder().build().unwrap();
  let register_receiver = merfolk_frontend_register::Register::builder().build().unwrap();
  register_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_receiver.register("add", |(a, b)| add(a, b)).unwrap();

  let (to, from): (Sender<merfolk_backend_in_process::InProcessChannel>, Receiver<merfolk_backend_in_process::InProcessChannel>) = channel(1);

  let merfolk_caller = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().to(to).build().unwrap())
    .frontend(register_caller)
    .build()
    .unwrap();

  let _merfolk_receiver = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().from(from).build().unwrap())
    .frontend(register_receiver)
    .build()
    .unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result: i32 = merfolk_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result, a + b);
}

#[test]
fn register_in_process_duplex() {
  use tokio::sync::mpsc::{channel, Receiver, Sender};

  let register_first = merfolk_frontend_register::Register::builder().build().unwrap();
  let register_second = merfolk_frontend_register::Register::builder().build().unwrap();

  register_first.register("add", |(a, b)| add(a, b)).unwrap();
  register_second.register("add", |(a, b)| add(a, b)).unwrap();

  let (to_first, from_first): (Sender<merfolk_backend_in_process::InProcessChannel>, Receiver<merfolk_backend_in_process::InProcessChannel>) = channel(1);
  let (to_second, from_second): (Sender<merfolk_backend_in_process::InProcessChannel>, Receiver<merfolk_backend_in_process::InProcessChannel>) = channel(1);

  let merfolk_first = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().to(to_first).from(from_second).build().unwrap())
    .frontend(register_first)
    .build()
    .unwrap();

  let merfolk_second = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().to(to_second).from(from_first).build().unwrap())
    .frontend(register_second)
    .build()
    .unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result_first: i32 = merfolk_first.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result_first, a + b);

  let (x, y) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result_second: i32 = merfolk_second.frontend(|f| f.call("add", &(x, y)).unwrap()).unwrap();
  assert_eq!(result_second, x + y);
}
