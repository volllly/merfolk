use crate::{
  interfaces::{Backend, Frontend},
  Mer, MerInit,
};

use crate::{Call, Reply};

use anyhow::Result;
use core::future::Future;

struct MockBackend {}

#[async_trait::async_trait(?Send)]
impl Backend for MockBackend {
  type Intermediate = String;

  async fn start(&mut self) -> Result<()> {
    Ok(())
  }
  async fn stop(&mut self) -> Result<()> {
    Ok(())
  }

  fn receiver<T, F>(&mut self, _receiver: T) -> Result<()>
  where
    T: Fn(Call<Self::Intermediate>) -> F + Send + Sync + 'static,
    F: Future<Output = Result<Reply<Self::Intermediate>>>
  {
    Ok(())
  }

  async fn call(&mut self, call: Call<Self::Intermediate>) -> Result<Reply<Self::Intermediate>> {
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

#[async_trait::async_trait(?Send)]
impl Frontend for MockFrontend {
  type Backend = MockBackend;
  type Intermediate = String;

  fn caller<T, F>(&mut self, _caller: T) -> Result<()>
  where
    T: Fn(Call<<Self::Backend as Backend>::Intermediate>) -> F + Send + Sync + 'static,
    F: Future<Output = Result<Reply<<Self::Backend as Backend>::Intermediate>>>
  {
    Ok(())
  }

  async fn receive(&self, call: Call<<Self::Backend as Backend>::Intermediate>) -> Result<Reply<<Self::Backend as Backend>::Intermediate>> {
    Ok(Reply { payload: call.payload })
  }
}

impl MockFrontend {
  fn call<T>(&self, call: T) -> T {
    call
  }
}

async fn setup<'a>() -> Mer<'a, MockBackend, MockFrontend> {
  MerInit {
    backend: MockBackend {},
    frontend: MockFrontend {},
  }
  .init().await
}

#[test]
fn init() {
  tokio_test::block_on(async { setup().await; });
}

#[test]
fn start() {
  tokio_test::block_on(async { setup().await.start().await.unwrap() });
}

#[test]
fn stop() {
  tokio_test::block_on(async { setup().await.stop().await.unwrap() });
}

#[test]
fn frontend() {
  let rnd: i32 = rand::random();
  tokio_test::block_on(async { setup().await.frontend(|f| assert_eq!(f.call(rnd), rnd)).await.unwrap() });
}

#[test]
fn backend() {
  tokio_test::block_on(async { setup().await.backend(|_b| ()).await.unwrap(); });
}
