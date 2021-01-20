use mer::*;

fn add(a: i32, b: i32) -> i32 {
  a + b
}

fn not_allowed() {}

#[test]
fn authentication_register_in_process() {
  use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
  };

  let register_caller = mer_frontend_register::RegisterInit {}.init();
  let register_receiver = mer_frontend_register::RegisterInit {}.init();
  register_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_receiver.register("add", |(a, b)| add(a, b)).unwrap();

  let (to, from): (Sender<mer_backend_in_process::InProcessChannel>, Receiver<mer_backend_in_process::InProcessChannel>) = mpsc::channel(1);

  let auth = (rand::random::<i32>().to_string(), rand::random::<i32>().to_string());
  let auth_cloned = (auth.0.clone(), auth.1.clone());

  let mer_caller = MerInit {
    backend: mer_backend_in_process::InProcessInit { to: to.into(), ..Default::default() }.init().unwrap(),
    frontend: register_caller,
    middlewares: Some(vec![mer_middleware_authentication::AuthenticationInit {
      auth: (auth.0, auth.1),
      ..Default::default()
    }.init_boxed()])
  }
  .init();

  let mut mer_receiver = MerInit {
    backend: mer_backend_in_process::InProcessInit {
      from: from.into(),
      ..Default::default()
    }
    .init()
    .unwrap(),
    frontend: register_receiver,
    middlewares: Some(vec![mer_middleware_authentication::AuthenticationInit {
      scopes: vec![("add".to_string(), "calc".to_string())].into(),
      authenticator: Some(Box::new(move |a: (String, String), s: Vec<String>| {
        if a.0 == auth_cloned.0 && a.1 == auth_cloned.1 && s.contains(&"calc".to_string()) {
          Ok(())
        } else {
          Err(anyhow::anyhow!("{:?}, {:?} != {:?}, {:?}", a, s, auth_cloned, vec!["calc"]))
        }
      })),
      ..Default::default()
    }.init_boxed()])
  }
  .init();

  mer_receiver.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result: i32 = mer_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result, a + b);
}



#[test]
#[should_panic]
fn authentication_register_in_process_failing() {
  use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
  };

  let register_caller = mer_frontend_register::RegisterInit {}.init();
  let register_receiver = mer_frontend_register::RegisterInit {}.init();
  register_caller.register("not_allowed", |()| not_allowed()).unwrap();
  register_receiver.register("not_allowed", |()| not_allowed()).unwrap();

  let (to, from): (Sender<mer_backend_in_process::InProcessChannel>, Receiver<mer_backend_in_process::InProcessChannel>) = mpsc::channel(1);

  let auth = (rand::random::<i32>().to_string(), rand::random::<i32>().to_string());
  let auth_cloned = (auth.0.clone(), auth.1.clone());

  let mer_caller = MerInit {
    backend: mer_backend_in_process::InProcessInit { to: to.into(), ..Default::default() }.init().unwrap(),
    frontend: register_caller,
    middlewares: Some(vec![mer_middleware_authentication::AuthenticationInit {
      auth: (auth.0, auth.1),
      ..Default::default()
    }.init_boxed()])
  }
  .init();

  let mut mer_receiver = MerInit {
    backend: mer_backend_in_process::InProcessInit {
      from: from.into(),
      ..Default::default()
    }
    .init()
    .unwrap(),
    frontend: register_receiver,
    middlewares: Some(vec![mer_middleware_authentication::AuthenticationInit {
      scopes: vec![("add".to_string(), "calc".to_string())].into(),
      authenticator: Some(Box::new(move |a: (String, String), s: Vec<String>| {
        if a.0 == auth_cloned.0 && a.1 == auth_cloned.1 && s.contains(&"calc".to_string()) {
          Ok(())
        } else {
          Err(anyhow::anyhow!("{:?}, {:?} != {:?}, {:?}", a, s, auth_cloned, vec!["calc"]))
        }
      })),
      ..Default::default()
    }.init_boxed()])
  }
  .init();

  mer_receiver.start().unwrap();

  let _: () = mer_caller.frontend(|f| f.call("not_allowed", &()).unwrap()).unwrap();
}
