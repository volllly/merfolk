#[cfg(not(feature = "threadsafe"))]
use alloc::rc::Rc;
#[cfg(feature = "threadsafe")]
use alloc::sync::Arc;

#[cfg(not(feature = "threadsafe"))]
use core::cell::RefCell;

#[cfg(all(not(feature = "std"), feature = "treadsafe"))]
pub type SmatrtPointerMutex<T> = spin::Mutex<T>;
#[cfg(all(feature = "std", feature = "threadsafe"))]
pub type SmatrtPointerMutex<T> = std::sync::Mutex<T>;

#[cfg(feature = "threadsafe")]
pub struct SmartPointer<T>(pub Arc<SmatrtPointerMutex<T>>);
#[cfg(not(feature = "threadsafe"))]
pub struct SmartPointer<T>(pub Rc<RefCell<T>>);

unsafe impl<T> Send for SmartPointer<T> {}

#[cfg(feature = "threadsafe")]
#[macro_export]
macro_rules! smart_pointer {
  ($x:expr) => {
    SmartPointer(alloc::sync::Arc::new(SmatrtPointerMutex::new($x)))
  };
}

#[cfg(not(feature = "threadsafe"))]
#[macro_export]
macro_rules! smart_pointer {
  ($x:expr) => {
    SmartPointer(Rc::new(RefCell::new($x)))
  };
}

#[macro_export]
macro_rules! clone {
  ($x:expr) => {
    SmartPointer($x.0.clone())
  };
}

#[cfg(all(feature = "threadsafe", feature = "std"))]
#[macro_export]
macro_rules! access {
  ($x:expr) => {
    $x.0.lock()
  };
}

#[cfg(all(feature = "threadsafe", not(feature = "std")))]
#[macro_export]
macro_rules! access {
  ($x:expr) => {
    Ok(*$x.0.lock())
  };
}

#[cfg(feature = "threadsafe")]
#[macro_export]
macro_rules! access_mut {
  ($x:expr) => {
    access!($x)
  };
}

#[cfg(not(feature = "threadsafe"))]
#[macro_export]
macro_rules! access {
  ($x:expr) => {
    $x.0.borrow()
  };
}

#[cfg(not(feature = "threadsafe"))]
#[macro_export]
macro_rules! access_mut {
  ($x:expr) => {
    $x.0.borrow_mut()
  };
}
