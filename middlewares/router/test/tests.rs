use merfolk::*;

fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[test]
fn router_authentication_register_in_process() {
  use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
  };

  let register_caller = merfolk_frontend_register::Register::builder().build().unwrap();
  let register_receiver = merfolk_frontend_register::Register::builder().build().unwrap();
  register_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_caller.register("math_add", |(a, b)| add(a, b)).unwrap();
  register_receiver.register("add", |(a, b)| add(a, b)).unwrap();

  let (to, from): (Sender<merfolk_backend_in_process::InProcessChannel>, Receiver<merfolk_backend_in_process::InProcessChannel>) = mpsc::channel(1);

  let auth = (rand::random::<i32>().to_string(), rand::random::<i32>().to_string());
  let auth_cloned = (auth.0.clone(), auth.1.clone());

  let merfolk_caller = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().to(to).build().unwrap())
    .frontend(register_caller)
    .middlewares(vec![merfolk_middleware_authentication::Authentication::builder().auth((auth.0, auth.1)).build_boxed().unwrap()])
    .build()
    .unwrap();

  let _merfolk_receiver = Mer::builder()
    .backend(merfolk_backend_in_process::InProcess::builder().from(from).build().unwrap())
    .frontend(register_receiver)
    .middlewares(vec![
      merfolk_middleware_router::Router::builder()
        .routes(vec![("math_(.*)".to_string(), "$1".to_string())])
        .build_boxed()
        .unwrap(),
      merfolk_middleware_authentication::Authentication::builder()
        .scopes(vec![("add".to_string(), "calc".to_string())])
        .authenticator(move |a: (String, String), s: Vec<String>| {
          if a.0 == auth_cloned.0 && a.1 == auth_cloned.1 && s.contains(&"calc".to_string()) {
            Ok(())
          } else {
            Err(anyhow::anyhow!("{:?}, {:?} != {:?}, {:?}", a, s, auth_cloned, vec!["calc"]))
          }
        })
        .build_boxed()
        .unwrap(),
    ])
    .build()
    .unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result_add: i32 = merfolk_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  let result_math_add: i32 = merfolk_caller.frontend(|f| f.call("math_add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result_add, a + b);
  assert_eq!(result_math_add, a + b);
}
