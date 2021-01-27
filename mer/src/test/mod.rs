use crate::{
  interfaces::{Backend, Frontend},
  Mer, MerInit,
};

use crate::{Call, Reply};

use anyhow::Result;

#[cfg(not(feature = "std"))]
use alloc::string::String;

struct MockBackend {}

impl Backend for MockBackend {
  type Intermediate = String;

  fn start(&mut self) -> Result<()> {
    Ok(())
  }
  fn stop(&mut self) -> Result<()> {
    Ok(())
  }

  fn register<T>(&mut self, _receiver: T) -> Result<()>
  where
    T: Fn(Call<Self::Intermediate>) -> Result<Reply<Self::Intermediate>> + Send + Sync + 'static,
  {
    Ok(())
  }

  fn call(&mut self, call: Call<Self::Intermediate>) -> Result<Reply<Self::Intermediate>> {
    Ok(Reply { payload: call.payload })
  }

  fn serialize<T: serde::Serialize>(from: &T) -> Result<Self::Intermediate> {
    serde_json::to_string(from).map_err(|e| e.into())
  }

  fn deserialize<'b, T>(from: &'b Self::Intermediate) -> Result<T>
  where
    T: for<'de> serde::Deserialize<'de>,
  {
    serde_json::from_str(from).map_err(|e| e.into())
  }
}

struct MockFrontend {}

impl Frontend for MockFrontend {
  type Backend = MockBackend;

  fn register<T>(&mut self, _caller: T) -> Result<()>
  where
    T: Fn(Call<<Self::Backend as Backend>::Intermediate>) -> Result<Reply<<Self::Backend as Backend>::Intermediate>> + Send + Sync + 'static,
  {
    Ok(())
  }

  fn receive(&self, call: Call<<Self::Backend as Backend>::Intermediate>) -> Result<Reply<<Self::Backend as Backend>::Intermediate>> {
    Ok(Reply { payload: call.payload })
  }
}

impl MockFrontend {
  fn call<T>(&self, call: T) -> T {
    call
  }
}

fn setup<'a>() -> Mer<'a, MockBackend, MockFrontend> {
  MerInit {
    backend: MockBackend {},
    frontend: MockFrontend {},
    middlewares: None,
  }
  .init()
  .unwrap()
}

#[test]
fn init() {
  setup();
}

#[test]
fn start() {
  setup().start().unwrap()
}

#[test]
fn stop() {
  setup().stop().unwrap()
}

#[test]
fn frontend() {
  let rnd: i32 = rand::random();
  setup().frontend(|f| assert_eq!(f.call(rnd), rnd)).unwrap()
}

#[test]
fn backend() {
  setup().backend(|_b| ()).unwrap();
}
