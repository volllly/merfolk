use crate::interfaces::backend;

pub struct Empty<'a> {
  started: bool,
  receiver: Option<&'a dyn Fn(&dyn backend::Call)>,
}

struct Receive {}

impl backend::Receive for Receive {}

impl Empty<'_> {
  pub fn new<'a>() -> Empty<'a> {
    Empty {
      started: false,
      receiver: None,
    }
  }

  fn encode<T>(&self) -> Result<String, backend::Error> {
    Ok(String::from("\"\""))
  }

  fn decode(&self) -> Result<(), backend::Error> {
    Ok(())
  }
}

impl<'a> backend::Backend<'a> for Empty<'a> {
  fn start(&mut self) -> Result<(), backend::Error> {
    Ok(self.started = true)
  }

  fn stop(&mut self) -> Result<(), backend::Error> {
    Ok(self.started = false)
  }

  fn receive(&mut self, receiver: &'a dyn Fn(&dyn backend::Call)) -> Result<(), backend::Error> {
    Ok(self.receiver = Some(receiver))
  }

  #[allow(unused_variables)]
  fn call(&mut self, call: &dyn backend::Call) -> Result<&dyn backend::Receive, backend::Error> {
    Ok(&Receive {})
  }
}
