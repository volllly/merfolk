#[cfg(feature = "threadsafe")]
#[allow(unused_macros)]
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
#[allow(unused_macros)]
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

#[allow(unused_macros)]
macro_rules! clone_pointer {
  ($x:expr) => {
    $x.clone()
  };
}
