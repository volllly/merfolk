use crate::interfaces::backend;

pub struct Empty {
  started: bool,
}

impl Default for Empty {
  fn default() -> Self {
    Empty { started: false }
  }
}

impl Empty {
  pub fn new() -> Empty {
    Empty::default()
  }

  pub fn trigger(self) {
    // self.receiver.unwrap()(&crate::Call {
    //   procedure: &"",
    //   payload: (),
    // });
  }
}

impl<'a> backend::Backend<'a> for Empty {
  type Intermediate = ();

  fn start(&mut self) -> Result<(), backend::Error> {
    self.started = true;
    Ok(())
  }

  fn stop(&mut self) -> Result<(), backend::Error> {
    self.started = false;
    Ok(())
  }

  // fn receiver<T>(&mut self, receiver: T) -> Result<(), backend::Error>
  // where
  //   T: Fn(&crate::Call<Self::Intermediate>) -> crate::Reply<Self::Intermediate>,
  //   T: 'a,
  //   Self::Intermediate: 'a,
  // {
  //   self.receiver = Some(Box::new(receiver));
  //   Ok(())
  // }

  #[allow(unused_variables)]
  fn call(
    &mut self,
    call: &crate::Call<Box<dyn erased_serde::Serialize>>,
  ) -> Result<crate::Reply<Self::Intermediate>, backend::Error> {
    println!(
      "{}: {}",
      call.procedure,
      serde_json::to_string(&call.payload).unwrap()
    );
    Ok(crate::Reply { payload: () })
  }

  fn serialize(&self, from: &dyn erased_serde::Serialize) -> &() {
    println!("{}", serde_json::to_string(from).unwrap());
    &()
  }
}
