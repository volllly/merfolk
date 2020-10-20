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

  empty
    .call(&backend::Call {
      procedure: &"",
      payload: &(),
    })
    .unwrap();
}

#[test]
#[cfg(feature = "frontends")]
fn empty_register() {
  use frontends::*;
  let mut mer = setup_empty();

  assert_eq!(true, mer.register("", &empty::Empty {}).is_ok());
}

#[test]
fn frontend_call() {
  fn add(a: i32, b: i32) -> i32 {
    a + b
  }

  impl<'a, T> Call<'a, T>
  where
    T: backend::Backend<'a>,
  {
    fn add(&self, a: i32, b: i32) -> i32 {
      backend::Call {
        procedure: "add",
        payload: 
      }
      let parameters: (i32, i32);
      add(parameters.0, parameters.0)
    }
  }
}
