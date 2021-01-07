#[cfg(all(feature = "std", feature = "threadsafe"))]
macro_rules! smart_lock_type {
  ($x:ty) => {
    alloc::sync::Arc<std::sync::Mutex<$x>>
  }
}

#[cfg(all(not(feature = "std"), feature = "threadsafe"))]
macro_rules! smart_lock_type {
  ($x:ty) => {
    alloc::sync::Arc<spin::Mutex<$x>>
  }
}

#[cfg(not(feature = "threadsafe"))]
macro_rules! smart_lock_type {
  ($x:ty) => {
    alloc::rc::Rc<core::cell::RefCell<$x>>
  }
}

#[cfg(all(feature = "std", feature = "threadsafe"))]
macro_rules! smart_lock {
  ($x:expr) => {
    alloc::sync::Arc::new(std::sync::Mutex::new($x))
  };
}

#[cfg(all(not(feature = "std"), feature = "threadsafe"))]
#[cfg(feature = "threadsafe")]
macro_rules! smart_lock {
  ($x:expr) => {
    alloc::sync::Arc::new(spin::Mutex::new($x))
  };
}

#[cfg(not(feature = "threadsafe"))]
macro_rules! smart_lock {
  ($x:expr) => {
    alloc::rc::Rc::new(core::cell::RefCell::new($x))
  };
}

macro_rules! clone_lock {
  ($x:expr) => {
    $x.clone()
  };
}

#[cfg(all(feature = "threadsafe", feature = "std"))]
macro_rules! access {
  ($x:expr) => {
    $x.lock()
  };
}

#[cfg(all(feature = "threadsafe", not(feature = "std")))]
macro_rules! access {
  ($x:expr) => {
    Ok(*$x.lock())
  };
}

#[cfg(feature = "threadsafe")]
macro_rules! access_mut {
  ($x:expr) => {
    access!($x)
  };
}

#[cfg(not(feature = "threadsafe"))]
macro_rules! access {
  ($x:expr) => {
    $x.borrow()
  };
}

#[cfg(not(feature = "threadsafe"))]
macro_rules! access_mut {
  ($x:expr) => {
    $x.borrow_mut()
  };
}
