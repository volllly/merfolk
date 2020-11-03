use super::*;

fn setup_empty<'a>() -> Mer<'a, backends::Empty, frontends::Empty> {
  Mer::new().with_backend(backends::Empty::new()).with_frontnd(frontends::Empty::new()).build()
}

#[test]
fn initializes() {
  setup_empty();
}

// #[test]
// #[cfg(feature = "backends")]
// fn empty_trigger_receive() {
//   use backend::*;
//   let mut empty = backends::Empty::new();

//   #[allow(unused_variables)]
//   empty
//     .receiver(|call: &Call<()>| {
//       println!("called");
//       Reply { payload: () }
//     })
//     .unwrap();

//   empty.trigger();
// }

#[test]
#[cfg(feature = "backends")]
fn empty_call() {
  use backend::*;
  let mut empty = backends::Empty::new();

  empty.call(&Call { procedure: &"", payload: Box::new(()) }).unwrap();
}

// #[test]
// #[cfg(feature = "frontends")]
// fn empty_register() {
//   use frontends::*;
//   let mut mer = setup_empty();

//   assert_eq!(true, mer.register("", &empty::Empty {}).is_ok());
// }

#[test]
fn frontend_call() {
  fn add(a: i32, b: i32) -> i32 {
    a + b
  }

  impl<'a> Caller<'a, backends::Empty, frontends::Empty> {
    fn add(&self, a: i32, b: i32) -> i32 {
      let reply = self.call(&Call { procedure: "add", payload: Box::new((a, b)) });
      // self.backend.deserialize();
      reply.unwrap().payload
    }
  }

  let mer = setup_empty();

  mer.call.add(1, 2);
}

#[test]
fn frontend_receive() {
  fn add(a: i32, b: i32) -> i32 {
    a + b
  }

  let mer = setup_empty();

  mer
    .receive(&Call {
      procedure: "add",
      payload: &serde_json::to_string(&(1i32, 2i32)).unwrap(),
    })
    .unwrap();
}
