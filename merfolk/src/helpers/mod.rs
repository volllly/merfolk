//! Helper types and functions for merfolk.

pub mod smart_lock;

#[doc(hidden)]
pub fn is_send<T: Send>() {}
#[doc(hidden)]
pub fn is_sync<T: Sync>() {}
#[doc(hidden)]
pub fn is_sized<T: Sized>() {}
