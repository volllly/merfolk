#[allow(unused_macros)]
macro_rules! smart_pointer_type {
  ($x:ty) => {
    alloc::sync::Arc<$x>
  }
}

#[allow(unused_macros)]
macro_rules! smart_pointer {
  ($x:expr) => {
    alloc::sync::Arc::new($x)
  };
}

#[allow(unused_macros)]
macro_rules! clone_pointer {
  ($x:expr) => {
    $x.clone()
  };
}
