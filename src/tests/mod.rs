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
fn empty_receive() {
  use backend::*;
  let mut empty = backends::Empty::new();

  #[allow(unused_variables)]
  empty
    .receive(&|call: &dyn backend::Call| println!("called"))
    .unwrap();
}

#[test]
#[cfg(feature = "backends")]
fn empty_call() {
  use backend::*;
  let mut empty = backends::Empty::new();

  struct Call {}
  impl backend::Call for Call {}

  empty.call(&Call {}).unwrap();
}
