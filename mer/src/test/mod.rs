use crate::interfaces::{Backend, Frontend};

use crate::{Call, Reply};

use anyhow::Result;

struct MockBackend {}
impl Backend for MockBackend {
  type Intermediate = String;

  fn start(&mut self) -> Result<()> {
    Ok(())
  }
  fn stop(&mut self) -> Result<()> {
    Ok(())
  }

  fn receiver<T>(&mut self, receiver: T) -> Result<()>
  where
    T: Fn(Call<Self::Intermediate>) -> Result<Reply<Self::Intermediate>> + Send + Sync + 'static {
      Ok(())
    }

  fn call(&mut self, call: Call<Self::Intermediate>) -> Result<Reply<Self::Intermediate>> {
    Ok(Reply { payload: "".to_string() })
  }

  fn serialize<T: serde::Serialize>(from: &T) -> Result<Self::Intermediate> {
    serde_json::to_string(from).map_err(|e| e.into())
  }

  fn deserialize<'b, T>(from: &'b Self::Intermediate) -> Result<T>
  where
    T: for<'de> serde::Deserialize<'de> {
      serde_json::from_str(from).map_err(|e| e.into())
    }
}

struct MockFrontend {}

impl Frontend for MockFrontend {
  type Backend = MockBackend;
  type Intermediate = String;

  fn caller<T>(&mut self, caller: T) -> Result<()>
  where
    T: Fn(Call<<Self::Backend as Backend>::Intermediate>) -> Result<Reply<<Self::Backend as Backend>::Intermediate>> + Send + Sync + 'static {
      Ok(())
    }

  fn receive(&self, call: Call<<Self::Backend as Backend>::Intermediate>) -> Result<Reply<<Self::Backend as Backend>::Intermediate>> {
    Ok(Reply { payload: "".to_string() })
  }
}


#[test]
fn init() {

}
