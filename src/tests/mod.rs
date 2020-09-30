use super::*;

fn setup_empty<'a>() -> Mer<'a, backends::Empty<'a>> {
  Mer::new().with_backend(backends::Empty::new()).build()
}

#[test]
fn initializes() {
  setup_empty();
}

#[test]
#[cfg(feature = "backends")]
fn empty_trigger_receive() {
  use backend::*;
  let mut empty = backends::Empty::new();

  #[allow(unused_variables)]
  empty
    .receiver(|call: &backend::Call<()>| {
      println!("called");
      backend::Reply { payload: () }
    })
    .unwrap();

  empty.trigger();
}

#[test]
#[cfg(feature = "backends")]
fn empty_call() {
  use backend::*;
  let mut empty = backends::Empty::new();

  empty.call(&backend::Call { procedure: &"", payload: &() }).unwrap();
}
