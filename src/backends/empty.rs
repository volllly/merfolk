use crate::interfaces::backend;

pub struct Empty<'a> {
  started: bool,
  receiver: Option<&'a dyn Fn(&dyn backend::Call)>,
}

struct Receive {}

impl backend::Receive for Receive {}

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

  #[allow(dead_code)]
  fn encode<T>(&self) -> Result<String, backend::Error> {
    Ok(String::from("\"\""))
  }

  #[allow(dead_code)]
  fn decode(&self) -> Result<(), backend::Error> {
    Ok(())
  }
}

impl<'a> backend::Backend<'a> for Empty<'a> {
  fn start(&mut self) -> Result<(), backend::Error> {
    self.started = true;
    Ok(())
  }

  fn stop(&mut self) -> Result<(), backend::Error> {
    self.started = false;
    Ok(())
  }

  fn receive(&mut self, receiver: &'a dyn Fn(&dyn backend::Call)) -> Result<(), backend::Error> {
    self.receiver = Some(receiver);
    Ok(())
  }

  #[allow(unused_variables)]
  fn call(&mut self, call: &dyn backend::Call) -> Result<&dyn backend::Receive, backend::Error> {
    Ok(&Receive {})
  }
}
