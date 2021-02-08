use mer_frontend_derive::*;

#[test]
fn test_generic() {
  #[frontend]
  struct Data<T>
  where
    T: std::ops::Add<Output = T> + for<'de> serde::Deserialize<'de> + serde::Serialize + Copy + Send,
  {
    pub offset: T,
  }

  #[frontend(target = "Data")]
  trait Service<T>
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
}

#[test]
fn test() {
  #[frontend]
  struct Data {}

  #[frontend(target = "Data")]
  trait Service {
    fn add(a: i32, b: i32) -> i32 {
      a + b
    }

    fn empty() {}

    fn itentity(a: i32) -> i32 {
      a
    }
  }
}

#[test]
fn test_with_data() {
  #[frontend]
  struct Data {
    pub offset: i32,
  }

  #[frontend(target = "Data")]
  trait Service {
    fn add(a: i32, b: i32) -> i32 {
      a + b
    }

    fn add_with_offset(&self, a: i32, b: i32) -> i32 {
      a + b + self.offset
    }
  }
}
