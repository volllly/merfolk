use merfolk::*;
use merfolk_backend_http::Http;
use merfolk_frontend_register::Register;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
#[test]
fn te() {

  // build the backend
  let backend = Http::builder()
    // configure backend as client
    .speak("http://localhost:8080".parse::<hyper::Uri>().unwrap())
    // configure backend as server
    .listen(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081))
    .build()
    .unwrap();
  // build the client frontend
  let caller_frontend = Register::builder().build().unwrap();
  // build the server frontend
  // combine the frontends using the [`Duplex`](/merfolk_frontend_derive) frontend
  // build router middleware
  // build merfolk instance acting as client and server
  let merfolk = Mer::builder().frontend(Register::builder().build().unwrap()).backend(backend).build().unwrap();
  // call remote procedures via the caller frontend
  let result: String = merfolk.frontend(|f| f.caller.call("some_remote_function", &()).unwrap()).unwrap();
}
