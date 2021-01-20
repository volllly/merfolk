macro_rules! smart_lock_type {
  ($x:ty) => {
    alloc::sync::Arc<tokio::sync::Mutex<$x>>
  }
}

macro_rules! smart_lock {
  ($x:expr) => {
    alloc::sync::Arc::new(tokio::sync::Mutex::new($x))
  };
}


macro_rules! clone_lock {
  ($x:expr) => {
    $x.clone()
  };
}

macro_rules! access {
  ($x:expr) => {
    $x.lock()
  };
}

