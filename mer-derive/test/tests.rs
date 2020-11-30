use mer_derive::*;

#[test]
fn test() {
  struct Data<T> {
    pub offset: T
  }

  #[receiver(data = "Data")]
  trait Receiver<T> where T: std::ops::Add<Output = T> + for<'de> serde::Deserialize<'de> + serde::Serialize + Copy {
    fn add(a: T, b: T) -> T::Output {
      a + b
    }
  
    fn add_with_offset(&self, a: T, b: T) -> T::Output {
      a + b + self.offset
    }
  }

  struct Data2 {
    pub offset: i32
  }

  #[receiver(data = "Data2")]
  trait Receiver2 {
    fn add(a: i32, b: i32) -> i32 {
      a + b
    }
  
    fn add_with_offset(&self, a: i32, b: i32) -> i32 {
      a + b + self.offset
    }
  }
}
