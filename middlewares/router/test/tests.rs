use mer::*;

fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[test]
fn router_authentication_register_in_process() {
  use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
  };

  let register_caller = mer_frontend_register::RegisterInit {}.init();
  let register_receiver = mer_frontend_register::RegisterInit {}.init();
  register_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_caller.register("math_add", |(a, b)| add(a, b)).unwrap();
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
    }
    .init_boxed()]),
  }
  .init()
  .unwrap();

  let mut mer_receiver = MerInit {
    backend: mer_backend_in_process::InProcessInit {
      from: from.into(),
      ..Default::default()
    }
    .init()
    .unwrap(),
    frontend: register_receiver,
    middlewares: Some(vec![
      mer_middleware_router::RouterInit {
        routes: vec![("math_(.*)".to_string(), "$1".to_string())],
      }
      .init_boxed(),
      mer_middleware_authentication::AuthenticationInit {
        scopes: vec![("add".to_string(), "calc".to_string())].into(),
        authenticator: Some(Box::new(move |a: (String, String), s: Vec<String>| {
          if a.0 == auth_cloned.0 && a.1 == auth_cloned.1 && s.contains(&"calc".to_string()) {
            Ok(())
          } else {
            Err(anyhow::anyhow!("{:?}, {:?} != {:?}, {:?}", a, s, auth_cloned, vec!["calc"]))
          }
        })),
        ..Default::default()
      }
      .init_boxed(),
    ]),
  }
  .init()
  .unwrap();

  mer_receiver.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result_add: i32 = mer_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  let result_math_add: i32 = mer_caller.frontend(|f| f.call("math_add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result_add, a + b);
  assert_eq!(result_math_add, a + b);
}
