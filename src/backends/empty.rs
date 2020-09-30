use crate::interfaces::backend;

pub struct Empty<'a> {
  started: bool,
  receiver: Option<Box<dyn Fn(&backend::Call<()>) -> backend::Reply<()> + 'a>>,
}

impl Default for Empty<'_> {
  fn default() -> Self {
    Empty {
      started: false,
      receiver: None,
    }
  }
}

impl Empty<'_> {
  pub fn new<'a>() -> Empty<'a> {
    Empty::default()
  }

  pub fn trigger(self) {
    self.receiver.unwrap()(&backend::Call {
      procedure: &"",
      payload: &(),
    });
  }
}

impl<'a> backend::Backend<'a> for Empty<'a> {
  type Intermediate = ();

  fn start(&mut self) -> Result<(), backend::Error> {
    self.started = true;
    Ok(())
  }

  fn stop(&mut self) -> Result<(), backend::Error> {
    self.started = false;
    Ok(())
  }

  fn receiver<T>(&mut self, receiver: T) -> Result<(), backend::Error>
  where
    T: Fn(&backend::Call<Self::Intermediate>) -> backend::Reply<Self::Intermediate>,
    T: 'a,
    Self::Intermediate: 'a,
  {
    self.receiver = Some(Box::new(receiver));
    Ok(())
  }

  #[allow(unused_variables)]
  fn call(
    &mut self,
    call: &backend::Call<Self::Intermediate>,
  ) -> Result<&backend::Reply<Self::Intermediate>, backend::Error> {
    Ok(&backend::Reply { payload: () })
  }
}
