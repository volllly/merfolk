use mer::*;

#[test]
fn derive_in_process() {
  use std::sync::mpsc;

  #[mer_frontend_derive::frontend()]
  struct Data<T>
  where
    T: std::ops::Add<Output = T> + for<'de> serde::Deserialize<'de> + serde::Serialize + Copy + Send,
  {
    pub offset: T,
  }

  #[mer_frontend_derive::frontend(target = "Data")]
  trait Receiver<T>
  where
    T: std::ops::Add<Output = T> + for<'de> serde::Deserialize<'de> + serde::Serialize + Copy + Send,
  {
    fn add(a: T, b: T) -> T::Output {
      a + b
    }

    fn add_with_offset(&self, a: T, b: T) -> T::Output {
      a + b + self.offset
    }
  }

  let (to, from): (mpsc::Sender<mer_backend_in_process::InProcessChannel>, mpsc::Receiver<mer_backend_in_process::InProcessChannel>) = mpsc::channel();

  let mer_caller = MerInit {
    backend: mer_backend_in_process::InProcessInit { to: to.into(), ..Default::default() }.init().unwrap(),
    frontend: DataInit::<i32> { offset: 32 }.init(),
  }
  .init();

  let mut mer_register = MerInit {
    backend: mer_backend_in_process::InProcessInit {
      from: from.into(),
      ..Default::default()
    }
    .init()
    .unwrap(),
    frontend: DataInit::<i32> { offset: 32 }.init(),
  }
  .init();

  mer_register.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  assert_eq!(mer_caller.frontend(|f| { f.add(a, b).unwrap() }).unwrap(), a + b);
}
