#[cfg(not(feature = "threadsafe"))]
use alloc::rc::Rc;
#[cfg(feature = "threadsafe")]
use alloc::sync::Arc;

#[cfg(feature = "threadsafe")]
macro_rules! smart_pointer_type {
  ($x:ty) => {
    alloc::sync::Arc<$x>
  }
}

#[cfg(not(feature = "threadsafe"))]
macro_rules! smart_pointer_type {
  ($x:ty) => {
    alloc::rc::Rc<$x>
  }
}

#[cfg(feature = "threadsafe")]
macro_rules! smart_pointer {
  ($x:expr) => {
    alloc::sync::Arc::new($x)
  };
}

#[cfg(not(feature = "threadsafe"))]
macro_rules! smart_pointer {
  ($x:expr) => {
    alloc::rc::Rc::new($x)
  };
}

macro_rules! clone_pointer {
  ($x:expr) => {
    $x.clone()
  };
}
