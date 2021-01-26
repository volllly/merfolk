use anyhow::Result;

use crate::Reply;

#[macro_use]
pub mod smart_lock;

#[macro_use]
pub mod smart_pointer;

pub fn is_send<T: Send>() {}
pub fn is_sync<T: Sync>() {}
pub fn is_sized<T: Sized>() {}


#[cfg(any(feature = "c_ffi", feature = "cpp_ffi"))]
pub fn result_as_bool<T>(result: anyhow::Result<T>) -> bool {
  match result {
    Ok(_) => true,
    Err(_) => false
  }
}
