# merfolk

[![CI](https://github.com/volllly/merfolk/workflows/CI/badge.svg?branch=main)](https://github.com/volllly/merfolk/actions?query=workflow%3ACI)
[![docs](https://docs.rs/merfolk/badge.svg)](https://docs.rs/merfolk/)

[`merfolk`] is a **m**inimal **e**xtensible **r**emote procedure call **f**ramew**o**r**k**.

The architecture is split into three modular parts: the [`Backend`], the [`Frontend`] and optional [`Middleware`]s.

[`merfolk`] is a collection of parts. The main part is [`Mer`](crate::Mer) the orchestrator type and a [collection](#provided-modules) of [`Backend`]s, the [`Frontend`]s and [`Middleware`]s (the `Folk`).

[`Mer`] can act as a server or a client or both depending on the configuration.

### [`Backend`]
The Backend is responsible for sending and receiving RPCs. Depending on the [`Backend`] this can happen over different channels (e.g. http, serial port, etc.).
The [`Backend`] serializes and deserializes the RPCs using the [`serde`] framework.

### Frontend
The [`Frontend`] is providing an API to make RPCs and to receive them. The way RPCs are made by the client and and handled the server depend on the frontend [`Frontend`]

### Middleware
A [`Middleware`] can modify sent and received RPCs and replies. Or perform custom actions on a sent or received RPC and reply.

## Use [`Mer`]
[`Mer`] needs a [`Backend`] and a [`Frontend`] to operate.
The following examples uses the [`Http`](/merfolk_backend_http) [`Backend`] and the [`Register`](/merfolk_frontend_register) and [`Derive`](/merfolk_frontend_derive) [`Frontend`] (see their documentation on how to use them).

How to use [`Mer`] (how to setup the server and client) depends strongly on the used [`Frontend`].

### Server
```rust
// remote procedure definitions
fn add(a: i32, b: i32) -> i32 {
  a + b
}
fn subtract(a: i32, b: i32) -> i32 {
  a - b
}

// build the backend
let backend = Http::builder()
  // configure backend as server
  .listen(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080))
  .build()
  .unwrap();

// build the frontend
let frontend = Register::builder()
  .procedures(
    vec![
      ("add", Register::<Http>::make_procedure(|(a, b)| add(a, b))),
      ("subtract", Register::<Http>::make_procedure(|(a, b)| subtract(a, b))),
    ]
    .into_iter()
    .collect(),
  )
  .build()
  .unwrap();

// register the procedures in the frontend
frontend.register("add", |(a, b)| add(a, b)).unwrap();
frontend.register("subtract", |(a, b)| subtract(a, b)).unwrap();

// build merfolk instance acting as server
let _merfolk = Mer::builder().backend(backend).frontend(frontend).build().unwrap();
```

### Client
```rust
// build the backend
let backend = Http::builder()
  // configure backend as client
  .speak("http://localhost:8080".parse::<hyper::Uri>().unwrap())
  .build()
  .unwrap();

// build the frontend
let frontend = Register::builder().build().unwrap();

// build merfolk instance acting as client
let merfolk = Mer::builder().backend(backend).frontend(frontend).build().unwrap();

// call remote procedures via the frontend
let result_add: Result<i32> = merfolk.frontend(|f| f.call("add", &(1, 2))).unwrap();
let result_subtract: Result<i32> = merfolk.frontend(|f| f.call("subtract", &(1, 2))).unwrap();
```

## Advanced
```rust
// remote procedure definitions for server
#[frontend()]
struct Receiver {}

#[frontend(target = "Receiver")]
trait Definition {
  fn some_function(arg: String) {}
}

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
let receiver_frontend = Receiver::builder().build().unwrap();

// combine the frontends using the [`Duplex`](/merfolk_frontend_derive) frontend
let frontend = Duplex::builder().caller(caller_frontend).receiver(receiver_frontend).build().unwrap();

// build router middleware
let middleware = Router::builder().routes(vec![("prefix_(.*)".to_string(), "$1".to_string())]).build_boxed().unwrap();

// build merfolk instance acting as client and server
let merfolk = Mer::builder().backend(backend).frontend(frontend).middlewares(vec![middleware]).build().unwrap();

// call remote procedures via the caller frontend
let result: String = merfolk.frontend(|f| f.caller.call("some_remote_function", &()).unwrap()).unwrap();
```

## Provided Modules
| Type           | Name                                              | Description |
|----------------|---------------------------------------------------|---|
| [`Backend`]    | [`Http`](/merfolk_backend_http)                        | Communicates via Http and in `json` format.                                                                              |
| [`Backend`]    | [`InProcess`](/merfolk_backend_in_process)             | Communicates via [`tokio`](tokio) [`channels`](tokio::sync::mpsc::channel) in `json` format (mostly used for testing purposes). |
| [`Backend`]    | [`SerialPort`](/merfolk_backend_serialport)            | Communicates via serial port (using the [`serialport`](serialport) library) in [`ron`](ron) format.                                          |
| [`Frontend`]   | [`Derive`](/merfolk_frontend_derive)                   | Provides derive macros to derive a frontend from trait definitions.                                                      |
| [`Frontend`]   | [`Duplex`](/merfolk_frontend_duplex)                   | Allows for different frontends for calling and receiving RPCs.                                                            |
| [`Frontend`]   | [`Logger`](/merfolk_frontend_logger)                   | Provides a frontend using the [`log`] facade on the client side.                                                         |
| [`Frontend`]   | [`Register`](/merfolk_frontend_register)                 | Allows for manually registering procedures on the server side and calling any procedure on the client side.              |
| [`Middleware`] | [`Authentication`](/merfolk_middleware_authentication) | Adds simple authentication and scopes.                                                                                   |
| [`Middleware`] | [`Router`](/merfolk_middleware_router)                 | Adds simple routing of procedures based on the procedure name.                                                           |



## Develop a Module for [`Mer`] (called a `Folk`)
If communication over a specific channel or a different frontend etc. is needed a module can be created by implementing the [`Backend`], [`Frontend`] or [`Middleware`] trait.

For examples please see the [provided modules](#provided-modules)

[`Backend`]: interfaces::Backend
[`Frontend`]: interfaces::Frontend
[`Middleware`]: interfaces::Middleware
[`Mer`]: crate::Mer
[`merfolk`]: crate
