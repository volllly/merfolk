use crate::interfaces::backend;

type Receiver<'a> = dyn Fn(&crate::Call<()>) -> crate::Reply<()> + 'a;
pub struct Empty<'a> {
  started: bool,
  receiver: Option<Box<Receiver<'a>>>,
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
    self.receiver.unwrap()(&crate::Call {
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
    T: Fn(&crate::Call<Self::Intermediate>) -> crate::Reply<Self::Intermediate>,
    T: 'a,
    Self::Intermediate: 'a,
  {
    self.receiver = Some(Box::new(receiver));
    Ok(())
  }

  #[allow(unused_variables)]
  fn call(
    &mut self,
    call: &crate::Call<Self::Intermediate>,
  ) -> Result<&crate::Reply<Self::Intermediate>, backend::Error> {
    Ok(&crate::Reply { payload: () })
  }
}
