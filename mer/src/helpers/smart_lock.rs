#![allow(unused_macros)]

#[cfg(feature = "std")]
macro_rules! smart_lock_type {
  ($x:ty) => {
    alloc::sync::Arc<std::sync::Mutex<$x>>
  }
}

#[cfg(not(feature = "std"))]
macro_rules! smart_lock_type {
  ($x:ty) => {
    alloc::sync::Arc<spin::Mutex<$x>>
  }
}

#[cfg(feature = "std")]
macro_rules! smart_lock {
  ($x:expr) => {
    alloc::sync::Arc::new(std::sync::Mutex::new($x))
  };
}

#[cfg(not(feature = "std"))]
macro_rules! smart_lock {
  ($x:expr) => {
    alloc::sync::Arc::new(spin::Mutex::new($x))
  };
}

macro_rules! clone_lock {
  ($x:expr) => {
    $x.clone()
  };
}

#[cfg(feature = "std")]
macro_rules! access {
  ($x:expr) => {
    $x.lock()
  };
}

#[cfg(not(feature = "std"))]
macro_rules! access {
  ($x:expr) => {
    Ok::<spin::mutex::MutexGuard<_>, core::convert::Infallible>($x.lock())
  };
}
