//! The [`SmartLock`] type can be created and accessed with macros.
//! Its internal structure depends on the `std` feature.
//!
//! The type is an alias for [`alloc::sync::Arc`]`<`[`std::sync::Mutex`]`<T>>` in `std` contexts and [`alloc::sync::Arc`]`<`[`spin::Mutex`]`<T>>` in `no_std` contexts.
//!
//! # Example
//! ```
//! # extern crate alloc;
//! # use merfolk::{smart_lock, access, clone_lock, helpers::smart_lock::SmartLock};
//!
//! let lock: SmartLock<String> = smart_lock!("Hello".to_string());
//!
//! *access!(lock).unwrap() += ", World!";
//!
//! let lock_clone = clone_lock!(lock);
//! println!("{}", access!(lock).unwrap()); // Hello, World!
//! ```

#[cfg(feature = "std")]
//#[doc(cfg(feature = "std"))]
#[macro_export]
/// Expands to the Type [`alloc::sync::Arc`]`<`[`std::sync::Mutex`]`<T>>`.
macro_rules! smart_lock_type {
  ($x:ty) => {
    alloc::sync::Arc<std::sync::Mutex<$x>>
  }
}

#[cfg(not(feature = "std"))]
//#[doc(cfg(not(feature = "std")))]
#[macro_export]
/// Expands to the Type [`alloc::sync::Arc`]`<`[`spin::Mutex`]`<T>>`.
macro_rules! smart_lock_type {
  ($x:ty) => {
    alloc::sync::Arc<spin::Mutex<$x>>
  }
}

/// Type alias for the macro [`smart_lock_type`].
pub type SmartLock<T> = smart_lock_type!(T);

#[cfg(feature = "std")]
#[macro_export]
/// Create a [`SmartLock`].
macro_rules! smart_lock {
  ($x:expr) => {
    alloc::sync::Arc::new(std::sync::Mutex::new($x))
  };
}

#[cfg(not(feature = "std"))]
#[macro_export]
/// Create a [`SmartLock`].
macro_rules! smart_lock {
  ($x:expr) => {
    alloc::sync::Arc::new(spin::Mutex::new($x))
  };
}

#[macro_export]
/// Clone a [`SmartLock`].
macro_rules! clone_lock {
  ($x:expr) => {
    $x.clone()
  };
}

#[cfg(feature = "std")]
#[macro_export]
/// Access a [`SmartLock`].
macro_rules! access {
  ($x:expr) => {
    $x.lock()
  };
}

#[cfg(not(feature = "std"))]
#[macro_export]
/// Access a [`SmartLock`].
macro_rules! access {
  ($x:expr) => {
    Ok::<spin::mutex::MutexGuard<_>, core::convert::Infallible>($x.lock())
  };
}
