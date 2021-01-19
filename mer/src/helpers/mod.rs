#[macro_use]
pub mod smart_lock;

#[macro_use]
pub mod smart_pointer;

pub fn is_send<T: Send>() {}
pub fn is_sync<T: Sync>() {}
pub fn is_sized<T: Sized>() {}
