
use crate::interfaces::backend;
use serde::Deserialize;
use serde_json;

pub struct Empty {}

impl backend::Backend for Empty {
  type Encoded = String;

  #[allow(unused_variables)]
  fn encode<T>(&self, tx: backend::Tx<T>) -> Result<String, backend::Error> {
    Ok(String::from("\"\""))
  }

  fn decode<'de, T: Deserialize<'de>>(&self, rx: &'de Self::Encoded) -> Result<backend::Rx<T>, backend::Error> {
    Ok(backend::Rx::<T> {
      procedure: &"",
      payload: serde_json::from_str::<'de, T>(&rx).map_err(|e| backend::Error::Decode(e.to_string()))?
    })
  }
}
