#[derive(Debug)]
pub enum Error {
  Decode(String),
}

pub trait Receive {}

pub trait Call {}

pub trait Backend<'a> {
  fn start(&mut self) -> Result<(), Error>;
  fn stop(&mut self) -> Result<(), Error>;

  fn receive(&mut self, receiver: &'a dyn Fn(&dyn Call)) -> Result<(), Error>;
  fn call(&mut self, call: &dyn Call) -> Result<&dyn Receive, Error>;
}
