use super::Backend;

use anyhow::Result;

pub trait Middleware: Send {
  type Backend: Backend;

  fn wrap_call(&self, call: Result<crate::Call<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Call<<Self::Backend as Backend>::Intermediate>>;
  fn wrap_reply(&self, call: Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>;

  fn unwrap_call(&self, reply: Result<crate::Call<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Call<<Self::Backend as Backend>::Intermediate>>;
  fn unwrap_reply(&self, reply: Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>) -> Result<crate::Reply<<Self::Backend as Backend>::Intermediate>>;
}
