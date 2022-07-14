use merfolk::*;

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

  let register_first_caller = merfolk_frontend_register::Register::builder().build().unwrap();
  let register_first_receiver = merfolk_frontend_register::Register::builder().build().unwrap();

  register_first_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_first_receiver.register("sub", |(a, b)| sub(a, b)).unwrap();

  let register_second_caller = merfolk_frontend_register::Register::builder().build().unwrap();
  let register_second_receiver = merfolk_frontend_register::Register::builder().build().unwrap();

  register_second_caller.register("sub", |(a, b)| sub(a, b)).unwrap();
  register_second_receiver.register("add", |(a, b)| add(a, b)).unwrap();

  let (to_first, from_first): (Sender<merfolk_backend_in_process::InProcessChannel>, Receiver<merfolk_backend_in_process::InProcessChannel>) = mpsc::channel(1);
  let (to_second, from_second): (Sender<merfolk_backend_in_process::InProcessChannel>, Receiver<merfolk_backend_in_process::InProcessChannel>) = mpsc::channel(1);

  let merfolk_first = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().to(to_first).from(from_second).build().unwrap())
    .frontend(
      merfolk_frontend_duplex::Duplex::builder()
        .caller(register_first_caller)
        .receiver(register_first_receiver)
        .build()
        .unwrap(),
    )
    .build()
    .unwrap();

  let merfolk_second = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().to(to_second).from(from_first).build().unwrap())
    .frontend(
      merfolk_frontend_duplex::Duplex::builder()
        .caller(register_second_caller)
        .receiver(register_second_receiver)
        .build()
        .unwrap(),
    )
    .build()
    .unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result_first: i32 = merfolk_first.frontend(|f| f.caller.call("add", &(a, b)).unwrap()).unwrap();
  let result_second: i32 = merfolk_second.frontend(|f| f.caller.call("sub", &(a, b)).unwrap()).unwrap();

  assert_eq!(result_first, a + b);
  assert_eq!(result_second, a - b);
}
